// Design-bundle stand-in for `#/lib/story/book/pdf`. The real module pulls the whole
// PDF stack (@react-pdf, pdf-lib, fontkit, pdfkit, yoga, brotli: ~4 MB unminified)
// into `_ds_bundle.js`, which then exceeds the 12 MB upload cap. Only the export
// sheet's click handler reaches it (a lazy `import()` in story/export/run.ts); no card
// renders through it. Generating a PDF from a design is out of scope, so `toPdf`
// rejects with the product's own error type.
import { PdfStoryError } from "#/lib/story/book/pdfError";

export type PaperSize = "A4" | "LETTER" | "A5";
export { PdfStoryError };

export async function toPdf(): Promise<never> {
    throw new PdfStoryError("", new Error("PDF export isn't available in this preview."));
}

export function writeOutline(): void {}
