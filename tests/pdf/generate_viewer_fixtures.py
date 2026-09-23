"""Generate small synthetic PDFs used by the browser regression tests."""
from pathlib import Path
import io
from reportlab.pdfgen import canvas
from pypdf import PdfReader, PdfWriter

root = Path(__file__).resolve().parents[2] / "apps/web/tests/fixtures"
data = io.BytesIO()
pdf = canvas.Canvas(data, pagesize=(300, 400))
for text in ("FENIX first page", "SECOND invoice 123.45", "THIRD appendix"):
    pdf.drawString(30, 300, text)
    pdf.linkURL("https://untrusted.invalid/track", (20, 280, 280, 330))
    pdf.showPage()
pdf.save()
writer = PdfWriter(clone_from=PdfReader(io.BytesIO(data.getvalue())))
writer.add_js("app.alert('PDF JavaScript must never run')")
writer.write(root / "pdf-viewer.pdf")
writer.encrypt("fenix-test-password")
writer.write(root / "pdf-password.pdf")
