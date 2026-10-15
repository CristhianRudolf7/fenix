"""Offline, bounded PDF/image extraction. Invoked by a local supervised Fenix process.

stdout is NDJSON; third-party diagnostics go to stderr. No LLM, URLs, shell,
workspace writes, external OCR fallback, or automatic runtime model downloads.
"""
from __future__ import annotations

import contextlib
import errno
import hashlib
import io
import json
import math
import os
from pathlib import Path
import re
import stat
import sys
import warnings

MAX_BYTES = 64 * 1024 * 1024
MAX_PIXELS = 12_000_000
MAX_CHARS = 60_000
MAX_BLOCKS = 2000
MODEL_ROOT = Path("/opt/fenix-ocr/models")
MODELS = ("PP-OCRv6_small_det", "PP-OCRv6_small_rec")


class ExtractionError(Exception):
    pass


def validate(request):
    if not isinstance(request, dict) or set(request) != {"path", "sha256", "mode", "start_page", "page_count", "dpi"}:
        raise ExtractionError("Invalid extraction request")
    if not isinstance(request["path"], str) or not Path(request["path"]).is_absolute():
        raise ExtractionError("Expected an absolute local input path")
    if not isinstance(request["sha256"], str) or not re.fullmatch(r"[a-f0-9]{64}", request["sha256"]):
        raise ExtractionError("Expected a SHA-256 content precondition")
    if request["mode"] not in ("auto", "native", "ocr"):
        raise ExtractionError("Invalid extraction mode")
    for key, low, high in (("start_page", 1, 2000), ("page_count", 1, 20), ("dpi", 100, 300)):
        if type(request[key]) is not int or not low <= request[key] <= high:
            raise ExtractionError(f"Invalid {key}")


def snapshot(path, expected):
    before = os.lstat(path)
    if stat.S_ISLNK(before.st_mode) or getattr(before, "st_file_attributes", 0) & getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", 0):
        raise OSError(errno.ELOOP, "Input cannot be a symbolic link or reparse point", path)
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_NONBLOCK", 0) | getattr(os, "O_BINARY", 0)
    fd = os.open(path, flags)
    with os.fdopen(fd, "rb") as stream:
        meta = os.fstat(stream.fileno())
        if (meta.st_dev, meta.st_ino) != (before.st_dev, before.st_ino):
            raise ExtractionError("Document changed while opening")
        if not stat.S_ISREG(meta.st_mode) or meta.st_nlink != 1 or meta.st_size > MAX_BYTES:
            raise ExtractionError("Input must be a regular unlinked file up to 64 MiB")
        data = stream.read(MAX_BYTES + 1)
    if len(data) > MAX_BYTES or hashlib.sha256(data).hexdigest() != expected:
        raise ExtractionError("Document changed or exceeds the byte limit")
    return data


def emit(event, **fields):
    print(json.dumps({"event": event, **fields}, ensure_ascii=False, allow_nan=False), flush=True)


def usable(text):
    chars = [c for c in text if not c.isspace()]
    return bool(chars) and sum(c == "\ufffd" or not c.isprintable() for c in chars) / len(chars) < 0.02


class OCR:
    def __init__(self):
        self.engine = None

    def recognize(self, image):
        if self.engine is None:
            # Paths belong to the immutable operator image, never to a request.
            for name in MODELS:
                directory = MODEL_ROOT / name
                if not (directory / "inference.yml").is_file() or not (directory / "inference.pdiparams").is_file():
                    raise ExtractionError("OCR models missing; install the Fenix OCR execution image")
            os.environ.update(PADDLE_PDX_DISABLE_MODEL_SOURCE_CHECK="True", HF_HUB_OFFLINE="1",
                              OMP_NUM_THREADS="2", OPENBLAS_NUM_THREADS="2", MKL_NUM_THREADS="2")
            from paddleocr import PaddleOCR
            self.engine = PaddleOCR(
                # Paddle 3.3.1 + these v6 models hit an unsupported oneDNN PIR
                # attribute on CPU. Keep the verified plain CPU execution path.
                device="cpu", cpu_threads=2, enable_mkldnn=False,
                text_detection_model_name=MODELS[0],
                text_detection_model_dir=str(MODEL_ROOT / MODELS[0]),
                text_recognition_model_name=MODELS[1],
                text_recognition_model_dir=str(MODEL_ROOT / MODELS[1]),
                use_doc_orientation_classify=False, use_doc_unwarping=False,
                use_textline_orientation=False,
            )
        import numpy as np
        # Paddle image arrays use BGR; PIL is RGB.
        result = next(iter(self.engine.predict(np.asarray(image)[:, :, ::-1].copy())))
        texts, scores, boxes = result["rec_texts"], result["rec_scores"], result["rec_polys"]
        blocks = []
        for text, score, box in zip(texts, scores, boxes, strict=True):
            score = float(score)
            if not math.isfinite(score):
                raise ExtractionError("Invalid recognition score")
            points = [[float(x) / image.width, float(y) / image.height] for x, y in box]
            blocks.append({"text": str(text), "score": score, "polygon": points})
        return blocks


def page_output(number, source, blocks, alerts, state="complete"):
    used = 0
    emitted = 0
    low = False
    for block in blocks[:MAX_BLOCKS]:
        text = block["text"]
        if used + len(text) > MAX_CHARS:
            alerts.append("page_text_limit")
            state = "partial"
            break
        used += len(text)
        low |= block.get("score", 1) < 0.8
        # Small records remain recoverable through cursor-paginated process logs.
        for at in range(0, len(text), 2000):
            emit("block", page=number, source=source, index=emitted,
                 **{**block, "text": text[at:at+2000]})
            emitted += 1
    if len(blocks) > MAX_BLOCKS:
        alerts.append("page_block_limit")
        state = "partial"
    if low:
        alerts.append("low_recognition_score_not_a_calibrated_probability")
    if source == "ocr":
        alerts.append("reading_order_and_table_structure_not_verified")
        alerts.append("orientation_and_handwriting_not_verified")
        if not blocks:
            alerts.append("no_text_detected_does_not_prove_blank_page")
            state = "needs_review"
    emit("page", page=number, source=source, status=state, blocks=emitted, warnings=alerts)
    return state


def extract_pdf(data, request, ocr):
    import pypdfium2 as pdfium
    import pypdfium2.raw as raw
    try:
        doc = pdfium.PdfDocument(data)
    except pdfium.PdfiumError as exc:
        raise ExtractionError("PDF cannot be opened (invalid, encrypted, or unsupported)") from exc
    with doc:
        total = len(doc)
        start = request["start_page"] - 1
        if total > 2000 or start >= total:
            raise ExtractionError("PDF page count or requested range exceeds limits")
        end = min(total, start + request["page_count"])
        emit("document", pages=total, sha256=request["sha256"], selected_pages=[start+1, end],
             model=list(MODELS), coordinate_system="normalized_top_left_for_ocr; native_blocks_have_page_only")
        states = []
        for index in range(start, end):
            emit("progress", page=index+1, stage="extracting")
            try:
                with contextlib.closing(doc[index]) as page:
                    width, height = page.get_size()
                    scale = request["dpi"] / 72
                    if not all(math.isfinite(x) and x > 0 for x in (width, height)):
                        raise ExtractionError("Invalid page dimensions")
                    with contextlib.closing(page.get_textpage()) as textpage:
                        count = textpage.count_chars()
                        if count > MAX_CHARS:
                            raise ExtractionError("Native text exceeds page character limit")
                        text = textpage.get_text_bounded()
                    # Conservatively OCR image-bearing pages; a header alone must
                    # never cause the scanned body to be silently skipped.
                    has_images = any(True for _ in page.get_objects(filter=[raw.FPDF_PAGEOBJ_IMAGE], max_depth=16))
                    needs_ocr = request["mode"] == "ocr" or has_images or not usable(text)
                    if request["mode"] == "native" or not needs_ocr:
                        alerts = []
                        if has_images or not usable(text):
                            alerts.append("native_only_images_or_unreadable_text_not_extracted")
                        state = page_output(index+1, "native", [{"text": text}] if text else [], alerts,
                                            "partial" if alerts else "complete")
                    else:
                        if math.ceil(width*scale) * math.ceil(height*scale) > MAX_PIXELS:
                            raise ExtractionError("Rendered page exceeds pixel limit; use lower dpi")
                        try:
                            with contextlib.closing(page.render(scale=scale)) as bitmap:
                                image = bitmap.to_pil().convert("RGB")
                            with image, contextlib.redirect_stdout(sys.stderr):
                                blocks = ocr.recognize(image)
                            state = page_output(index+1, "ocr", blocks, ["mixed_page_full_ocr"] if usable(text) else [])
                        except Exception:
                            if usable(text):
                                page_output(index+1, "native", [{"text": text}], ["ocr_failed_native_text_only"], "partial")
                            raise
                    states.append(state)
            except Exception as exc:
                # Do not echo arbitrary exception messages containing document text.
                emit("page_error", page=index+1, code=type(exc).__name__,
                     message=str(exc) if isinstance(exc, ExtractionError) else "Page extraction failed")
                states.append("failed")
        return states, end+1 if end < total else None


def extract_image(data, request, ocr):
    from PIL import Image, ImageOps
    if request["start_page"] != 1:
        raise ExtractionError("Images contain one page")
    if request["mode"] == "native":
        raise ExtractionError("Images require OCR; use auto or ocr")
    Image.MAX_IMAGE_PIXELS = MAX_PIXELS
    with warnings.catch_warnings():
        warnings.simplefilter("error", Image.DecompressionBombWarning)
        with Image.open(io.BytesIO(data)) as source:
            if source.width * source.height > MAX_PIXELS or getattr(source, "n_frames", 1) != 1:
                raise ExtractionError("Image exceeds pixel limit or has multiple frames")
            if source.format not in ("PNG", "JPEG"):
                raise ExtractionError("Only PDF, PNG and JPEG inputs are supported")
            image = ImageOps.exif_transpose(source).convert("RGB")
    emit("document", pages=1, selected_pages=[1, 1], sha256=request["sha256"], model=list(MODELS),
         coordinate_system="normalized_top_left_after_exif_orientation")
    with image, contextlib.redirect_stdout(sys.stderr):
        blocks = ocr.recognize(image)
    return [page_output(1, "ocr", blocks, [])], None


def run(request, ocr=None):
    validate(request)
    data = snapshot(request["path"], request["sha256"])
    ocr = ocr or OCR()
    if data.startswith(b"%PDF-"):
        states, next_page = extract_pdf(data, request, ocr)
    else:
        states, next_page = extract_image(data, request, ocr)
    complete = all(state == "complete" for state in states)
    emit("summary", status="complete" if complete else "partial", pages_processed=len(states),
         next_page=next_page, sha256=request["sha256"], scope="selected_pages_only",
         untrusted_content=True)
    return 0 if complete else 2


if __name__ == "__main__":
    try:
        if len(sys.argv) != 2 or len(sys.argv[1]) > 16000:
            raise ExtractionError("Expected one bounded JSON request")
        raise SystemExit(run(json.loads(sys.argv[1])))
    except Exception as error:
        emit("error", code=type(error).__name__,
             message=str(error) if isinstance(error, ExtractionError) else "Document extraction failed")
        raise SystemExit(2)
