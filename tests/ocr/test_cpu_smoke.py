"""Opt-in actual model test on synthetic text; not an accuracy benchmark."""
import contextlib
import hashlib
import io
import json
import os
from pathlib import Path
import tempfile
import unittest
from test_worker import worker


@unittest.skipUnless(os.environ.get("FENIX_OCR_SMOKE_MODELS"), "Set FENIX_OCR_SMOKE_MODELS to pinned offline models")
class CPUSmokeTests(unittest.TestCase):
    def test_actual_cpu_recognition_on_image_and_scanned_pdf(self):
        from PIL import Image, ImageDraw, ImageFont
        old_root = worker.MODEL_ROOT
        worker.MODEL_ROOT = Path(os.environ["FENIX_OCR_SMOKE_MODELS"])
        try:
            with tempfile.TemporaryDirectory() as directory:
                image = Image.new("RGB", (1200, 400), "white")
                font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 42)
                draw = ImageDraw.Draw(image)
                draw.text((40, 60), "Factura FENIX 2026", font=font, fill="black")
                draw.text((40, 140), "Total: 123.45 EUR", font=font, fill="black")
                engine = worker.OCR()
                for suffix in ("png", "pdf"):
                    path = Path(directory)/("invoice."+suffix)
                    image.save(path)
                    request = dict(path=str(path), sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
                                   mode="auto", start_page=1, page_count=1, dpi=100)
                    output = io.StringIO()
                    with contextlib.redirect_stdout(output): code = worker.run(request, engine)
                    records = [json.loads(line) for line in output.getvalue().splitlines()]
                    text = " ".join(r["text"] for r in records if r["event"] == "block")
                    self.assertEqual(code, 0)
                    self.assertIn("FENIX 2026", text)
                    self.assertIn("123.45", text)
                    self.assertTrue(all(r["source"] == "ocr" for r in records if r["event"] == "block"))
        finally: worker.MODEL_ROOT = old_root


if __name__ == "__main__": unittest.main()
