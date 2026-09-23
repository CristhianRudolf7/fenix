"""Real PDF parsing with deterministic OCR doubles; no model/network dependency."""
import contextlib
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("ocr_worker", ROOT / "scripts/ocr/worker.py")
worker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(worker)


def make_pdf(pages):
    """Tiny independent PDF fixture writer: (text, has_image, width, height)."""
    objects = [b"", b"", b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>"]
    page_refs = []
    for text, image, width, height in pages:
        content = f"BT /F1 16 Tf 20 100 Td ({text}) Tj ET".encode()
        resources = "/Font << /F1 3 0 R >>"
        if image:
            image_id = len(objects) + 1
            objects.append(b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length 3 >>\nstream\n\xff\xff\xff\nendstream")
            resources += f" /XObject << /I1 {image_id} 0 R >>"
            content += b"\nq 200 0 0 200 0 0 cm /I1 Do Q"
        stream_id = len(objects)+1
        objects.append(f"<< /Length {len(content)} >>\nstream\n".encode()+content+b"\nendstream")
        page_refs.append(len(objects)+1)
        objects.append(f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {width} {height}] /Resources << {resources} >> /Contents {stream_id} 0 R >>".encode())
    objects[0] = b"<< /Type /Catalog /Pages 2 0 R >>"
    objects[1] = f"<< /Type /Pages /Count {len(pages)} /Kids [{' '.join(f'{p} 0 R' for p in page_refs)}] >>".encode()
    data = b"%PDF-1.4\n"; offsets = [0]
    for n, obj in enumerate(objects, 1):
        offsets.append(len(data)); data += f"{n} 0 obj\n".encode()+obj+b"\nendobj\n"
    xref = len(data)
    data += f"xref\n0 {len(offsets)}\n0000000000 65535 f \n".encode()
    data += b"".join(f"{x:010} 00000 n \n".encode() for x in offsets[1:])
    return data+f"trailer\n<< /Size {len(offsets)} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n".encode()


class FakeOCR:
    def __init__(self, fail=False): self.calls = 0; self.fail = fail
    def recognize(self, image):
        self.calls += 1
        if self.fail: raise RuntimeError("private text must not leak in diagnostics")
        return [{"text": "Scanned body 123.45", "score": 0.99, "polygon": [[0, 0], [1, 0], [1, 1], [0, 1]]}]


class WorkerTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(); self.path = Path(self.temp.name)/"input.pdf"
    def tearDown(self): self.temp.cleanup()
    def request(self, pages=None, **options):
        self.path.write_bytes(make_pdf(pages or [("Invoice 123.45", False, 300, 300)]))
        return dict(path=str(self.path), sha256=hashlib.sha256(self.path.read_bytes()).hexdigest(),
                    mode="auto", start_page=1, page_count=5, dpi=200, **options)
    def run_worker(self, request, ocr=None):
        output = io.StringIO()
        with contextlib.redirect_stdout(output): code = worker.run(request, ocr or FakeOCR())
        return code, [json.loads(line) for line in output.getvalue().splitlines()]
    def test_native_text_skips_model_and_preserves_page_provenance(self):
        ocr = FakeOCR(); code, events = self.run_worker(self.request(), ocr)
        self.assertEqual(code, 0); self.assertEqual(ocr.calls, 0)
        block = next(x for x in events if x["event"] == "block")
        self.assertEqual(block["page"], 1); self.assertEqual(block["source"], "native")
        self.assertIn("123.45", block["text"])
    def test_mixed_page_does_not_lose_scanned_body_or_duplicate_header(self):
        ocr = FakeOCR(); code, events = self.run_worker(self.request([("Header", True, 300, 300)]), ocr)
        self.assertEqual(code, 0); self.assertEqual(ocr.calls, 1)
        self.assertEqual([x["text"] for x in events if x["event"] == "block"], ["Scanned body 123.45"])
    def test_native_mode_reports_incomplete_image_coverage(self):
        req = self.request([("Header", True, 300, 300)]); req["mode"] = "native"
        code, events = self.run_worker(req)
        self.assertEqual(code, 2); self.assertEqual(events[-1]["status"], "partial")
    def test_failed_ocr_preserves_native_text_but_never_reports_success(self):
        code, events = self.run_worker(self.request([("Header", True, 300, 300)]), FakeOCR(True))
        self.assertEqual(code, 2)
        self.assertIn("Header", str(events)); self.assertNotIn("private text", str(events))
    def test_page_selection_and_continuation(self):
        req = self.request([(f"Page {i}", False, 300, 300) for i in range(1, 4)])
        req.update(start_page=2, page_count=1)
        code, events = self.run_worker(req)
        self.assertEqual(code, 0); self.assertEqual(events[-1]["next_page"], 3)
        self.assertEqual([x["page"] for x in events if x["event"] == "page"], [2])
    def test_huge_page_is_rejected_before_render_or_ocr(self):
        ocr = FakeOCR(); code, events = self.run_worker(self.request([("", True, 100000, 100000)]), ocr)
        self.assertEqual(code, 2); self.assertEqual(ocr.calls, 0)
        self.assertIn("pixel limit", str(events))
    def test_hash_precondition_and_links(self):
        req = self.request(); self.path.write_bytes(b"changed")
        with self.assertRaises(worker.ExtractionError): self.run_worker(req)
        req = self.request(); other = self.path.with_name("hard.pdf"); os.link(self.path, other)
        with self.assertRaises(worker.ExtractionError): self.run_worker(req)
        other.unlink(); other.symlink_to(self.path); req["path"] = str(other)
        with self.assertRaises(OSError): self.run_worker(req)
    def test_invalid_parameters_and_url(self):
        for key, value in [("page_count", 21), ("start_page", 0), ("dpi", 301), ("dpi", True), ("path", "https://example.com/a.pdf"), ("sha256", "bad")]:
            req = self.request(); req[key] = value
            with self.assertRaises(worker.ExtractionError): self.run_worker(req)
    def test_corrupt_pdf_is_not_empty_success(self):
        req = self.request(); self.path.write_bytes(b"%PDF-corrupt")
        req["sha256"] = hashlib.sha256(self.path.read_bytes()).hexdigest()
        with self.assertRaises(worker.ExtractionError): self.run_worker(req)
    def test_image_ocr_and_empty_recognition(self):
        from PIL import Image
        req = self.request(); Image.new("RGB", (400, 200), "white").save(self.path, format="PNG")
        req["sha256"] = hashlib.sha256(self.path.read_bytes()).hexdigest()
        code, events = self.run_worker(req)
        self.assertEqual(code, 0); self.assertEqual(events[-1]["pages_processed"], 1)
        class Empty:
            def recognize(self, image): return []
        code, events = self.run_worker(req, Empty())
        self.assertEqual(code, 2); self.assertIn("needs_review", str(events))


if __name__ == "__main__": unittest.main()
