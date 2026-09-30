/**
 * THE BOOK AS ONE HTML DOCUMENT: the same blocks and the same stylesheet the
 * EPUB uses, in a single page with a table of contents. It is what a "read as
 * document" view and a print stylesheet will be built on (plan 3.2).
 */
import { storyLabel } from "./book";
import { bookCss, type FontFaces } from "./css";
import { type BookLabels, DEFAULT_LABELS, type RenderContext, sectionBody } from "./render";
import type { Book, ImageVariant, Typeface } from "./types";
import { el, serialize } from "./xml";

/**
 * The print sheet: book margins, the contents on a page of its own, a story
 * per page, no figure or choice split across a page, links printed as text,
 * and ink rather than the screen's greys.
 */
export const PRINT_CSS = `@media screen{body{max-width:44em;margin:0 auto;padding:0 5%}}
@page{size:auto;margin:18mm 16mm}
@media print{body{margin:0;color:#000;background:#fff}nav.toc{break-after:page}.story{break-before:page}.story:first-of-type{break-before:auto}figure,.scene,.choice .options{break-inside:avoid}h1,h2,.story-head{break-after:avoid}a{color:inherit;text-decoration:none}.narr,.eyebrow,.synopsis,.scene-name,.arm-label,.choice-label{opacity:1;color:#333}img{max-width:100%}}
`;

export interface HtmlTheme {
    typeface: Typeface;
    /** Font files for `@font-face`, or null to name the family only. */
    fonts?: FontFaces | null;
    labels?: BookLabels;
    /** The `src` for an asset path (a scene background is asked for as a `thumb`); default is the path itself. */
    imageSrc?: (path: string, variant?: ImageVariant) => string | null;
}

export function toHtml(book: Book, theme: HtmlTheme): string {
    const labels = theme.labels ?? DEFAULT_LABELS;
    const colors = new Set<string>();
    const multi = book.parts.length > 1;
    let n = 0;
    const toc: ReturnType<typeof el>[] = [];
    const body = book.parts.map((part) => {
        const items: ReturnType<typeof el>[] = [];
        const sections = part.sections.map((s) => {
            n += 1;
            const ctx: RenderContext = { options: book.meta.options, labels, idPrefix: `s${n}-`, imageSrc: theme.imageSrc ?? ((p) => p), colors };
            items.push(el("li", {}, el("a", { href: `#s${n}-top` }, storyLabel(s))));
            return sectionBody(s, ctx, "h2");
        });
        toc.push(multi ? el("li", {}, part.title, el("ol", {}, items)) : el("li", {}, items));
        return multi ? el("div", { class: "part", id: `part-${part.id}` }, el("h1", {}, part.title), sections) : sections;
    });
    // A one-story document needs no contents.
    const nav = n > 1 ? el("nav", { class: "toc" }, el("h2", {}, labels.contents), el("ol", {}, multi ? toc : toc.flatMap((li) => li.children))) : null;
    const css = bookCss(theme.typeface, theme.fonts ?? null, colors) + PRINT_CSS;
    const doc = el(
        "html",
        { lang: book.meta.language },
        el("head", {}, el("meta", { charset: "utf-8" }), el("meta", { name: "viewport", content: "width=device-width, initial-scale=1" }), el("title", {}, book.meta.title), el("style", {}, css)),
        el("body", {}, el("h1", { class: "book-title" }, book.meta.title), nav, body.flat()),
    );
    return `<!DOCTYPE html>\n${serialize(doc)}\n`;
}
