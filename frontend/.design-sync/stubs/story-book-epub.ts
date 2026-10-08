// Design-bundle stand-in for `#/lib/story/book/epub` (and, through it, fflate):
// only the export sheet's "Download EPUB" click reaches `toEpub`, and the bundle
// must stay under the 12 MiB upload cap. See story-book-pdf.ts.
export async function toEpub(): Promise<never> {
    throw new Error("EPUB export isn't available in this preview.");
}
