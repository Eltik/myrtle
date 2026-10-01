/** Kept out of `pdf.tsx` so the sheet can name a failed story without loading react-pdf. */
/** A story that could not be laid out: the sheet names it. */
export class PdfStoryError extends Error {
    constructor(
        readonly story: string,
        cause: unknown,
    ) {
        super(cause instanceof Error ? cause.message : String(cause));
        this.name = "PdfStoryError";
    }
}
