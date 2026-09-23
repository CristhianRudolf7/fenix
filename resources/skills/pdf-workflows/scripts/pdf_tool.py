#!/usr/bin/env python3
"""Fenix PDF operations. Run in the tenant execution image, never the controller.

Protocol and examples: ../references/operations.md. All paths are workspace-relative.
No downloads, subprocesses, external services or in-place changes.
"""
import argparse
from contextlib import closing, contextmanager
import hashlib
import io
import json
import math
import os
from pathlib import PurePosixPath
import secrets
import stat
import sys
import zipfile

MAX_BYTES = 64 * 1024 * 1024
MAX_PAGES = 100
MAX_PIXELS = 12_000_000


def require(condition, message):
    if not condition:
        raise ValueError(message)


def integer(value, low, high):
    require(type(value) is int and low <= value <= high, f"Expected integer {low}..{high}")
    return value


class Buffer(io.BytesIO):
    def write(self, value):
        require(self.tell() + len(value) <= MAX_BYTES, "Output exceeds 64 MiB")
        return super().write(value)


class Workspace:
    """Descriptor-relative traversal prevents symlink escapes and output clobbering."""
    def __init__(self, root):
        self.fd = os.open(root, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)

    def close(self):
        os.close(self.fd)

    @contextmanager
    def parent(self, path):
        require(isinstance(path, str) and 0 < len(path) <= 2048 and "\0" not in path,
                "Invalid workspace path")
        parts = PurePosixPath(path).parts
        require(parts and not path.startswith("/") and all(p not in ("..", ".") for p in parts)
                and ":" not in path and "\\" not in path, "Use a relative workspace file path")
        fd = os.dup(self.fd)
        try:
            for part in parts[:-1]:
                next_fd = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=fd)
                os.close(fd)
                fd = next_fd
            yield fd, parts[-1]
        finally:
            os.close(fd)

    def read(self, path, limit=MAX_BYTES):
        with self.parent(path) as (parent, name):
            fd = os.open(name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=parent)
            with os.fdopen(fd, "rb") as file:
                info = os.fstat(file.fileno())
                require(stat.S_ISREG(info.st_mode) and info.st_nlink == 1, "Input must be a regular file with one link")
                require(info.st_size <= limit, "Input exceeds size limit")
                data = file.read(limit + 1)
                require(len(data) <= limit, "Input exceeds size limit")
                return data

    def publish(self, path, data):
        require(len(data) <= MAX_BYTES, "Output exceeds 64 MiB")
        with self.parent(path) as (parent, name):
            temp = ".fenix-pdf-" + secrets.token_hex(16)
            fd = os.open(temp, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600, dir_fd=parent)
            try:
                with os.fdopen(fd, "wb") as file:
                    file.write(data)
                    file.flush()
                    os.fsync(file.fileno())
                # Atomic no-replace publication, including when target is a symlink.
                os.link(temp, name, src_dir_fd=parent, dst_dir_fd=parent, follow_symlinks=False)
            finally:
                os.unlink(temp, dir_fd=parent)
        return {"path": path, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}


def pages(value, count):
    result = list(range(1, count + 1)) if value is None else value
    require(isinstance(result, list) and 1 <= len(result) <= MAX_PAGES, "Select 1..100 pages")
    return [integer(p, 1, count) - 1 for p in result]


def rectangle(value):
    require(isinstance(value, list) and len(value) == 4
            and all(type(v) in (int, float) and math.isfinite(v) for v in value), "Invalid rectangle")
    x0, y0, x1, y1 = value
    require(0 <= x0 < x1 <= 1 and 0 <= y0 < y1 <= 1, "Rectangle must be normalized [left, top, right, bottom]")
    return value


def reader(data):
    from pypdf import PdfReader
    require(data.startswith(b"%PDF-"), "Input is not a PDF")
    pdf = PdfReader(io.BytesIO(data), strict=True)
    require(not pdf.is_encrypted, "Encrypted PDF: provide an authorized decrypted copy first")
    require(1 <= len(pdf.pages) <= 2000, "PDF must contain 1..2000 pages")
    return pdf


def serialize(writer):
    out = Buffer()
    writer.write(out)
    data = out.getvalue()
    check = reader(data)
    require(len(check.pages) == len(writer.pages), "Output page count verification failed")
    return data


def image(data):
    from PIL import Image, ImageOps
    Image.MAX_IMAGE_PIXELS = MAX_PIXELS
    source = Image.open(io.BytesIO(data))
    require(source.format in ("PNG", "JPEG") and getattr(source, "n_frames", 1) == 1, "Use a single PNG or JPEG")
    require(source.width * source.height <= MAX_PIXELS, "Image exceeds 12 million pixels")
    with source:
        rgba = ImageOps.exif_transpose(source).convert("RGBA")
        background = Image.new("RGB", rgba.size, "white")
        background.paste(rgba, mask=rgba.getchannel("A"))
        return background


def render(data, selected, dpi):
    import pypdfium2 as pdfium
    with pdfium.PdfDocument(data) as pdf:
        pdf.init_forms()
        for index in selected:
            with closing(pdf[index]) as page:
                width, height = page.get_size()
                scale = dpi / 72
                require(math.ceil(width * scale) * math.ceil(height * scale) <= MAX_PIXELS,
                        "Page exceeds 12 million pixels; reduce dpi")
                with closing(page.render(scale=scale)) as bitmap:
                    yield index, (width, height), bitmap.to_pil().convert("RGB")


def raster_pdf(data, selected, dpi, rectangles=None, grayscale=False):
    from PIL import ImageDraw, ImageOps
    from reportlab.pdfgen import canvas
    from reportlab.lib.utils import ImageReader
    out = Buffer()
    pdf = canvas.Canvas(out, pageCompression=1)
    for index, (width, height), bitmap in render(data, selected, dpi):
        if grayscale:
            bitmap = ImageOps.grayscale(bitmap)
        for box in (rectangles or {}).get(str(index + 1), []):
            x0, y0, x1, y1 = rectangle(box)
            ImageDraw.Draw(bitmap).rectangle((math.floor(x0 * bitmap.width), math.floor(y0 * bitmap.height),
                                             math.ceil(x1 * bitmap.width), math.ceil(y1 * bitmap.height)), fill=0)
        pdf.setPageSize((width, height))
        pdf.drawImage(ImageReader(bitmap), 0, 0, width, height)
        pdf.showPage()
        bitmap.close()
    pdf.save()
    return out.getvalue()


OPTIONS = {
    "inspect": {"pages"}, "assemble": set(), "split": {"pages"},
    "rotate": {"pages", "degrees"}, "crop": {"pages", "box"},
    "watermark": {"pages", "text"}, "number": {"pages", "start"},
    "metadata": {"values"}, "compress": set(), "fill": {"fields"},
    "render": {"pages", "dpi"}, "images": {"dpi"},
    "rasterize": {"pages", "dpi", "grayscale"}, "redact": {"rectangles", "dpi"},
}


def execute(workspace, request):
    from pypdf import PdfWriter
    require(isinstance(request, dict) and set(request) <= {"operation", "inputs", "output", "options"}, "Unknown request fields")
    operation = request.get("operation")
    require(isinstance(operation, str) and operation in OPTIONS, "Unsupported PDF operation")
    options = request.get("options", {})
    require(isinstance(options, dict) and set(options) <= OPTIONS[operation], "Unsupported operation options")
    inputs = request.get("inputs")
    require(isinstance(inputs, list) and 1 <= len(inputs) <= 16, "Provide 1..16 inputs")
    require(operation in ("assemble", "images") or len(inputs) == 1, "This operation needs exactly one input")
    snapshots, hashes = [], []
    for entry in inputs:
        require(isinstance(entry, dict) and set(entry) <= {"path", "sha256", "pages"} and "path" in entry, "Invalid input")
        require("pages" not in entry or operation == "assemble", "Input pages are only for assemble; use options.pages otherwise")
        data = workspace.read(entry["path"])
        digest = hashlib.sha256(data).hexdigest()
        require("sha256" not in entry or entry["sha256"] == digest, "Input SHA-256 changed")
        snapshots.append(data)
        require(sum(map(len, snapshots)) <= 128 * 1024 * 1024, "Combined inputs exceed 128 MiB")
        hashes.append({"path": entry["path"], "sha256": digest})
    pdfs = [] if operation == "images" else [reader(data) for data in snapshots]
    result = {"operation": operation, "inputs": hashes, "warnings": []}
    warnings = result["warnings"]
    if operation == "inspect":
        require("output" not in request, "Inspect does not write a file")
        pdf = pdfs[0]
        selected = pages(options.get("pages", list(range(1, min(len(pdf.pages), 20) + 1))), len(pdf.pages))
        fields = pdf.get_fields() or {}
        result.update(page_count=len(pdf.pages), metadata={str(k): str(v)[:2000] for k, v in (pdf.metadata or {}).items()},
                      fields=[{"name": name, "type": str(field.get("/FT", "")), "value": str(field.get("/V", ""))[:2000]}
                              for name, field in list(fields.items())[:200]],
                      fields_truncated=len(fields) > 200,
                      pages=[{"page": i + 1, "width": float(pdf.pages[i].mediabox.width),
                              "height": float(pdf.pages[i].mediabox.height), "rotation": pdf.pages[i].rotation}
                             for i in selected],
                      has_xfa=bool(pdf.trailer["/Root"].get("/AcroForm", {}).get_object().get("/XFA"))
                      if "/AcroForm" in pdf.trailer["/Root"] else False)
        return result
    output = request.get("output")
    extension = ".zip" if operation in ("split", "render") else ".pdf"
    require(isinstance(output, str) and output.lower().endswith(extension), f"Output must be a new {extension} file")
    # Validate the destination before expensive work. publish still checks atomically.
    with workspace.parent(output) as (parent, name):
        try:
            os.stat(name, dir_fd=parent, follow_symlinks=False)
        except FileNotFoundError:
            pass
        else:
            raise ValueError("Output already exists; choose a new path")
    if any(any(f.get("/FT") == "/Sig" for f in (p.get_fields() or {}).values()) for p in pdfs):
        warnings.append("This rewrite does not preserve digital signature validity.")
    dpi = integer(options.get("dpi", 150), 72, 300)
    writer = PdfWriter()
    if operation == "images":
        from reportlab.pdfgen import canvas
        from reportlab.lib.utils import ImageReader
        out = Buffer()
        doc = canvas.Canvas(out, pageCompression=1)
        for data in snapshots:
            with image(data) as bitmap:
                width, height = bitmap.width * 72 / dpi, bitmap.height * 72 / dpi
                doc.setPageSize((width, height))
                doc.drawImage(ImageReader(bitmap), 0, 0, width, height)
                doc.showPage()
        doc.save()
        final = out.getvalue()
    elif operation == "assemble":
        total = 0
        for index, (pdf, entry) in enumerate(zip(pdfs, inputs)):
            selected = pages(entry.get("pages"), len(pdf.pages))
            total += len(selected)
            require(total <= MAX_PAGES, "Output exceeds 100 pages")
            if pdf.get_fields():
                pdf.add_form_topname(f"source{index + 1}")
                warnings.append(f"Form fields from input {index + 1} use prefix source{index + 1}.")
            writer.append(pdf, pages=selected, import_outline=True)
        final = serialize(writer)
    else:
        pdf = pdfs[0]
        if operation in ("rotate", "crop", "watermark", "number", "metadata", "compress", "fill", "redact"):
            require(len(pdf.pages) <= MAX_PAGES, "Transform at most 100 pages; assemble a smaller document first")
        selected = pages(options.get("pages"), len(pdf.pages))
        if operation in ("render", "rasterize", "redact"):
            require(len(selected) <= 20, "Raster operations accept at most 20 pages")
        if operation in ("split", "render"):
            out = Buffer()
            with zipfile.ZipFile(out, "w", compression=zipfile.ZIP_DEFLATED) as archive:
                if operation == "split":
                    for position, index in enumerate(selected, 1):
                        part = PdfWriter()
                        part.append(pdf, pages=[index])
                        archive.writestr(f"{position:04d}-page-{index + 1}.pdf", serialize(part))
                else:
                    for position, (index, _, bitmap) in enumerate(render(snapshots[0], selected, dpi), 1):
                        png = Buffer()
                        bitmap.save(png, "PNG")
                        bitmap.close()
                        archive.writestr(f"{position:04d}-page-{index + 1}.png", png.getvalue())
            final = out.getvalue()
        elif operation in ("rasterize", "redact"):
            rectangles = options.get("rectangles", {})
            require(type(options.get("grayscale", False)) is bool, "grayscale must be boolean")
            if operation == "redact":
                require(isinstance(rectangles, dict) and rectangles, "Provide redaction rectangles by page")
                for key, boxes in rectangles.items():
                    require(key in {str(i + 1) for i in selected} and isinstance(boxes, list)
                            and 1 <= len(boxes) <= 100, "Invalid redaction page or rectangle count")
                    for box in boxes:
                        rectangle(box)
            final = raster_pdf(snapshots[0], selected, dpi, rectangles, options.get("grayscale", False))
            warnings.append("Raster reconstruction removes original text, vectors, forms, attachments, bookmarks and signatures; inspect visual quality and redaction coverage.")
        else:
            writer.clone_document_from_reader(pdf)
            if operation == "rotate":
                degrees = integer(options.get("degrees"), -270, 270)
                require(degrees in (-270, -180, -90, 90, 180, 270), "Use a nonzero multiple of 90 degrees")
                for index in set(selected):
                    writer.pages[index].rotate(degrees)
            elif operation == "crop":
                from pypdf.generic import RectangleObject
                x0, y0, x1, y1 = rectangle(options.get("box"))
                for index in set(selected):
                    page = writer.pages[index]
                    page.transfer_rotation_to_content()
                    box = page.cropbox
                    left, bottom, width, height = map(float, (box.left, box.bottom, box.width, box.height))
                    page.cropbox = RectangleObject((left + x0 * width, bottom + (1 - y1) * height,
                                                   left + x1 * width, bottom + (1 - y0) * height))
                warnings.append("Crop changes the visible page box; hidden content remains recoverable. It is not redaction.")
            elif operation in ("watermark", "number"):
                from reportlab.pdfgen import canvas
                from reportlab.pdfbase import pdfmetrics
                from reportlab.pdfbase.ttfonts import TTFont
                pdfmetrics.registerFont(TTFont("FenixSans", "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"))
                text = options.get("text", "")
                require(operation == "number" or isinstance(text, str) and 0 < len(text) <= 200, "Watermark needs 1..200 characters")
                start = integer(options.get("start", 1), 1, 1_000_000)
                for position, index in enumerate(dict.fromkeys(selected)):
                    page = writer.pages[index]
                    page.transfer_rotation_to_content()
                    box = page.cropbox
                    left, bottom, width, height = map(float, (box.left, box.bottom, box.width, box.height))
                    stamp = Buffer()
                    overlay = canvas.Canvas(stamp, pagesize=(float(page.mediabox.right), float(page.mediabox.top)))
                    label = text if operation == "watermark" else str(start + position)
                    size = min(32 if operation == "watermark" else 11,
                               width * .85 / max(pdfmetrics.stringWidth(label, "FenixSans", 1), 1))
                    overlay.setFont("FenixSans", size)
                    overlay.setFillColorRGB(.4, .4, .4)
                    overlay.drawCentredString(left + width / 2, bottom + (height / 2 if operation == "watermark" else min(18, height * .06)), label)
                    overlay.save()
                    page.merge_page(reader(stamp.getvalue()).pages[0])
            elif operation == "metadata":
                values = options.get("values", {})
                require(isinstance(values, dict) and set(values) <= {"Title", "Author", "Subject", "Keywords", "Creator"}
                        and all(isinstance(v, str) and len(v) <= 2000 for v in values.values()), "Invalid metadata")
                writer.metadata = None
                writer.root_object.pop("/Metadata", None)
                writer.add_metadata({"/" + k: v for k, v in values.items()})
                warnings.append("Updates document Info/XMP metadata only; this is not a forensic sanitization of all objects.")
            elif operation == "compress":
                writer.compress_identical_objects(remove_duplicates=True, remove_unreferenced=True)
                for page in writer.pages:
                    page.compress_content_streams()
                warnings.append("Lossless stream compression; file size may not decrease.")
            elif operation == "fill":
                fields = options.get("fields")
                available = pdf.get_fields() or {}
                require(isinstance(fields, dict) and 1 <= len(fields) <= 100 and set(fields) <= set(available)
                        and all(isinstance(v, str) and len(v) <= 2000 for v in fields.values()), "Unknown field or invalid form value")
                require(not pdf.trailer["/Root"].get("/AcroForm", {}).get_object().get("/XFA"), "XFA forms are unsupported")
                require(all(available[k].get("/FT") == "/Tx" for k in fields), "Only AcroForm text fields are supported")
                for page in writer.pages:
                    if page.get("/Annots"):
                        writer.update_page_form_field_values(page, fields, auto_regenerate=False)
            final = serialize(writer)
    if extension == ".pdf":
        result["page_count"] = len(reader(final).pages)
        if operation == "fill":
            actual = reader(final).get_form_text_fields() or {}
            require(all(actual.get(k) == v for k, v in options["fields"].items()), "Form value verification failed")
        if operation == "redact":
            verified = reader(final)
            require(all(not p.extract_text().strip() and not p.get("/Annots") for p in verified.pages)
                    and not verified.attachments and not verified.get_fields(), "Raster redaction verification failed")
            result["verification"] = "Fresh image-only PDF: no extractable text, form fields, annotations or attachments. Visual coverage still needs review."
    result["output"] = workspace.publish(output, final)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workspace", required=True)
    parser.add_argument("--request", required=True, help="Workspace-relative JSON request file")
    args = parser.parse_args()
    try:
        with closing(Workspace(args.workspace)) as workspace:
            request = json.loads(workspace.read(args.request, 64 * 1024))
            result = execute(workspace, request)
        print(json.dumps({"status": "completed", **result}, ensure_ascii=False))
    except Exception as exc:
        print(json.dumps({"status": "failed", "error": str(exc)[:1000]}))
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
