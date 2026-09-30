/**
 * THE BOOK AS AN EPUB 3.3, zipped with fflate as entries become ready.
 *
 * Layout: `mimetype` FIRST and STORED (the OCF rule every reading system
 * sniffs by), `META-INF/container.xml`, then `OEBPS/` with the package
 * document, `nav.xhtml` (a NESTED toc: part > operation > story, plus
 * landmarks), `toc.ncx` for EPUB 2 readers, one XHTML per story, the
 * stylesheet with the typeface EMBEDDED, the images deduplicated within the
 * book, a cover and a colophon. Every document is built by `xml.ts`, so story
 * text is only ever a text node.
 *
 * Nothing here touches the network or a canvas: fetching, resizing and the
 * cover are `EpubDeps`, so the structure is tested in Node with fakes and the
 * browser supplies the real ones (`browser.ts`).
 */
import { strToU8, Zip, ZipDeflate, ZipPassThrough } from "fflate";
import { storyLabel } from "./book";
import { bookCss, type FontFaces } from "./css";
import { type BookLabels, DEFAULT_LABELS, type RenderContext, sectionBody } from "./render";
import type { Block, Book, ImageVariant, Section, Typeface } from "./types";
import { el, type XElement, type XNode, xmlDocument } from "./xml";

export interface EpubImage {
    bytes: Uint8Array;
    mime: "image/jpeg" | "image/png";
}

export interface EpubFont {
    regular: Uint8Array;
    bold: Uint8Array;
    format: "woff" | "woff2";
}

export interface EpubDeps {
    /** Fetch and prepare one asset path; null leaves the picture out. */
    /** A `thumb` is a scene background at <= 600 px; a `full` image is a CG at <= 1,200 px. */
    loadImage: (path: string, signal?: AbortSignal, variant?: ImageVariant) => Promise<EpubImage | null>;
    loadFont: (typeface: Exclude<Typeface, "device">, signal?: AbortSignal) => Promise<EpubFont | null>;
    /** The composed cover, or null for a book without one. */
    cover: (book: Book, signal?: AbortSignal) => Promise<EpubImage | null>;
}

export interface EpubProgress {
    /** Stories written so far, of `total`. */
    done: number;
    total: number;
    /** The story being written, empty once the book is being closed. */
    label: string;
}

export interface EpubOptions {
    typeface: Typeface;
    labels?: BookLabels;
    /** `dcterms:modified`; defaults to now. */
    modified?: Date;
    signal?: AbortSignal;
    onProgress?: (p: EpubProgress) => void;
    /** Images fetched at once. */
    concurrency?: number;
    /**
     * Where the zip goes as it is written. With a sink every chunk is handed
     * over as fflate produces it and awaited before the next story starts, so
     * memory holds one story's images at a time; the result then has no Blob.
     */
    sink?: EpubSink;
}

/** A byte destination: a `FileSystemWritableFileStream`'s writer, or a test's fake. */
export interface EpubSink {
    write: (chunk: Uint8Array) => Promise<void>;
}

export interface EpubResult {
    /** The whole book, or null when it went to `sink`. */
    blob: Blob | null;
    /** Distinct images written, and asset paths that failed to load. */
    images: number;
    failedImages: string[];
    files: string[];
}

const XHTML_NS = "http://www.w3.org/1999/xhtml";
const OPS_NS = "http://www.idpf.org/2007/ops";

function abortIfNeeded(signal?: AbortSignal): void {
    if (signal?.aborted) throw signal.reason instanceof Error ? signal.reason : new DOMException("Export cancelled", "AbortError");
}

/** `2026-09-30T12:00:00Z`, the form `dcterms:modified` requires (no milliseconds). */
export function modifiedStamp(d: Date): string {
    return `${d.toISOString().slice(0, 19)}Z`;
}

const THUMB = "thumb:";

/** The key an image is written under: the path for a CG, `thumb:` and the path for a scene thumbnail, so the two sizes of one file never collide. */
export function imageKey(path: string, variant: ImageVariant = "full"): string {
    return variant === "thumb" ? `${THUMB}${path}` : path;
}

/** Every image a block list shows under the book's options, in order, as `imageKey`s. */
export function imagePaths(blocks: readonly Block[], images: Book["meta"]["options"]["images"]): string[] {
    const out: string[] = [];
    const visit = (list: readonly Block[]) => {
        for (const b of list) {
            if (b.kind === "figure" && images !== "none") for (const i of b.images) out.push(i.url);
            else if (b.kind === "scene" && images === "cg+bg" && b.url) out.push(imageKey(b.url, "thumb"));
            else if (b.kind === "choice") for (const arm of b.arms) visit(arm.blocks);
        }
    };
    visit(blocks);
    return out;
}

async function mapLimit<T>(items: readonly T[], limit: number, fn: (item: T) => Promise<void>): Promise<void> {
    let next = 0;
    const worker = async () => {
        while (next < items.length) {
            const item = items[next];
            next += 1;
            await fn(item);
        }
    };
    await Promise.all(Array.from({ length: Math.min(limit, items.length) }, worker));
}

function xhtml(lang: string, title: string, body: XNode[], bodyType?: string): string {
    return xmlDocument(el("html", { xmlns: XHTML_NS, "xmlns:epub": OPS_NS, "xml:lang": lang, lang }, el("head", {}, el("meta", { charset: "UTF-8" }), el("title", {}, title), el("link", { rel: "stylesheet", type: "text/css", href: "../css/book.css" })), el("body", { "epub:type": bodyType }, body)), "<!DOCTYPE html>");
}

/** A toc entry: a label, a target, and children. */
interface TocNode {
    label: string;
    href: string;
    children: TocNode[];
}

/**
 * Consecutive stories that share an operation code (a `_beg` and its `_end`)
 * nest under the operation, labelled by their phase; everything else is one
 * entry. A book of several groups nests all of that under one entry per group.
 */
function tocOf(book: Book, hrefOf: (s: Section) => string, partHref: (i: number) => string | null): TocNode[] {
    const partNodes = book.parts.map((part, pi) => {
        const nodes: TocNode[] = [];
        const list = part.sections;
        for (let i = 0; i < list.length; i += 1) {
            const s = list[i];
            const code = s.code?.trim();
            let j = i;
            while (code && j + 1 < list.length && list[j + 1].code?.trim() === code) j += 1;
            if (j > i && code) {
                nodes.push({ label: storyLabel(s), href: hrefOf(s), children: list.slice(i, j + 1).map((c) => ({ label: c.tag || c.name, href: hrefOf(c), children: [] })) });
                i = j;
            } else nodes.push({ label: storyLabel(s), href: hrefOf(s), children: [] });
        }
        return { label: part.title, href: partHref(pi) ?? nodes[0]?.href ?? "", children: nodes };
    });
    return book.parts.length > 1 ? partNodes : (partNodes[0]?.children ?? []);
}

function navList(nodes: TocNode[]): XElement {
    return el(
        "ol",
        {},
        nodes.map((n) => el("li", {}, el("a", { href: n.href }, n.label), n.children.length > 0 ? navList(n.children) : null)),
    );
}

function ncxPoints(nodes: TocNode[], counter: { n: number }): XElement[] {
    return nodes.map((n) => {
        counter.n += 1;
        const order = String(counter.n);
        return el("navPoint", { id: `np${order}`, playOrder: order }, el("navLabel", {}, el("text", {}, n.label)), el("content", { src: n.href }), ncxPoints(n.children, counter));
    });
}

function depth(nodes: TocNode[]): number {
    return nodes.length === 0 ? 0 : 1 + Math.max(...nodes.map((n) => depth(n.children)));
}

/** A file name inside the book for a story id: ASCII word characters only. */
function safeName(id: string): string {
    return id.replace(/[^A-Za-z0-9_-]/g, "_");
}

export async function toEpub(book: Book, opts: EpubOptions, deps: EpubDeps): Promise<EpubResult> {
    const { signal } = opts;
    const labels = opts.labels ?? DEFAULT_LABELS;
    const lang = book.meta.language;
    const options = book.meta.options;
    const sections = book.parts.flatMap((p) => p.sections);
    const files: string[] = [];
    const chunks: Uint8Array[] = [];
    let zipError: Error | null = null;
    const sink = opts.sink;
    let pending: Promise<void> = Promise.resolve();
    const zip = new Zip((err, data) => {
        if (err) zipError = err;
        else if (sink) pending = pending.then(() => sink.write(data));
        else chunks.push(data);
    });
    /** Wait for the sink to take everything written so far. */
    const flush = async () => {
        await pending;
        if (zipError) throw zipError;
    };
    const put = (path: string, data: Uint8Array | string, compress = true) => {
        abortIfNeeded(signal);
        const bytes = typeof data === "string" ? strToU8(data) : data;
        const entry = compress ? new ZipDeflate(path, { level: 9 }) : new ZipPassThrough(path);
        zip.add(entry);
        entry.push(bytes, true);
        files.push(path);
    };
    const manifest: { id: string; href: string; type: string; props?: string }[] = [];
    const spine: { idref: string; linear?: boolean }[] = [];

    put("mimetype", "application/epub+zip", false);
    put("META-INF/container.xml", xmlDocument(el("container", { version: "1.0", xmlns: "urn:oasis:names:tc:opendocument:xmlns:container" }, el("rootfiles", {}, el("rootfile", { "full-path": "OEBPS/content.opf", "media-type": "application/oebps-package+xml" })))));

    // The typeface, embedded. A font that fails to load leaves the family name, never a broken @font-face.
    let faces: FontFaces | null = null;
    if (opts.typeface !== "device") {
        const font = await deps.loadFont(opts.typeface, signal);
        if (font) {
            const ext = font.format;
            const type = ext === "woff2" ? "font/woff2" : "font/woff";
            put(`OEBPS/fonts/${opts.typeface}-400.${ext}`, font.regular, false);
            put(`OEBPS/fonts/${opts.typeface}-700.${ext}`, font.bold, false);
            manifest.push({ id: "font-400", href: `fonts/${opts.typeface}-400.${ext}`, type }, { id: "font-700", href: `fonts/${opts.typeface}-700.${ext}`, type });
            faces = { regular: `../fonts/${opts.typeface}-400.${ext}`, bold: `../fonts/${opts.typeface}-700.${ext}`, format: ext };
        }
    }

    // The cover.
    const cover = await deps.cover(book, signal);
    if (cover) {
        const ext = cover.mime === "image/png" ? "png" : "jpg";
        put(`OEBPS/images/cover.${ext}`, cover.bytes, false);
        manifest.push({ id: "cover-image", href: `images/cover.${ext}`, type: cover.mime, props: "cover-image" });
        put("OEBPS/text/cover.xhtml", xhtml(lang, labels.cover, [el("section", { class: "cover", "epub:type": "cover" }, el("img", { src: `../images/cover.${ext}`, alt: book.meta.title }))], "cover"));
        manifest.push({ id: "cover", href: "text/cover.xhtml", type: "application/xhtml+xml" });
        spine.push({ idref: "cover", linear: false });
    }
    spine.push({ idref: "nav" });

    // Images, each distinct path once, written before the story that first shows it.
    const written = new Map<string, string>();
    const failed: string[] = [];
    let imageCount = 0;
    const colors = new Set<string>();
    const hrefOf = (s: Section) => `text/${safeName(s.id)}.xhtml`;
    const multi = book.parts.length > 1;
    const partHrefs: (string | null)[] = [];

    /** Load one image by `imageKey` and write it, once per book; a failure is recorded, not thrown. */
    const loadKey = async (key: string): Promise<void> => {
        if (written.has(key) || failed.includes(key)) return;
        const variant: ImageVariant = key.startsWith(THUMB) ? "thumb" : "full";
        const path = variant === "thumb" ? key.slice(THUMB.length) : key;
        abortIfNeeded(signal);
        let image: EpubImage | null = null;
        try {
            image = await deps.loadImage(path, signal, variant);
        } catch (err) {
            if (signal?.aborted) throw err;
            image = null;
        }
        if (!image) {
            failed.push(key);
            return;
        }
        imageCount += 1;
        const name = `i${imageCount}.${image.mime === "image/png" ? "png" : "jpg"}`;
        put(`OEBPS/images/${name}`, image.bytes, false);
        manifest.push({ id: `img${imageCount}`, href: `images/${name}`, type: image.mime });
        written.set(key, `../images/${name}`);
    };

    let done = 0;
    for (const [pi, part] of book.parts.entries()) {
        if (multi) {
            // A part opens on its group's key visual, when the group has one.
            const art = book.meta.partArt?.[part.id];
            if (art && options.images !== "none") await loadKey(art);
            const artSrc = art ? written.get(art) : undefined;
            const href = `text/part-${pi + 1}.xhtml`;
            put(`OEBPS/${href}`, xhtml(lang, part.title, [el("section", { class: "part" }, artSrc ? el("figure", { class: "part-art" }, el("img", { src: artSrc, alt: "" })) : null, el("h1", {}, part.title))]));
            manifest.push({ id: `part${pi + 1}`, href, type: "application/xhtml+xml" });
            spine.push({ idref: `part${pi + 1}` });
            partHrefs.push(href);
        } else partHrefs.push(null);
        for (const section of part.sections) {
            abortIfNeeded(signal);
            opts.onProgress?.({ done, total: sections.length, label: storyLabel(section) });
            const wanted = [...new Set(imagePaths(section.blocks, options.images))].filter((p) => !written.has(p) && !failed.includes(p));
            await mapLimit(wanted, opts.concurrency ?? 4, loadKey);
            const ctx: RenderContext = { options, labels, idPrefix: "", imageSrc: (p, variant) => written.get(imageKey(p, variant)) ?? null, colors };
            const href = hrefOf(section);
            put(`OEBPS/${href}`, xhtml(lang, storyLabel(section), [sectionBody(section, ctx, "h1")], "bodymatter"));
            const id = `x${manifest.filter((m) => m.id.startsWith("x")).length + 1}`;
            manifest.push({ id, href, type: "application/xhtml+xml" });
            spine.push({ idref: id });
            await flush();
            done += 1;
        }
    }
    opts.onProgress?.({ done, total: sections.length, label: "" });

    // Colophon.
    const modified = modifiedStamp(opts.modified ?? new Date());
    put("OEBPS/text/colophon.xhtml", xhtml(lang, labels.colophon, [el("section", { class: "colophon", "epub:type": "colophon" }, el("h1", {}, book.meta.title), el("p", {}, labels.credit), el("p", {}, labels.rights), el("p", {}, labels.madeWith), el("p", {}, `${book.meta.identifier} · ${modified.slice(0, 10)}`))]));
    manifest.push({ id: "colophon", href: "text/colophon.xhtml", type: "application/xhtml+xml" });
    spine.push({ idref: "colophon" });

    // Stylesheet, now that every colour a span used is known.
    put("OEBPS/css/book.css", bookCss(opts.typeface, faces, colors));
    manifest.push({ id: "css", href: "css/book.css", type: "text/css" });

    // Navigation.
    const toc = tocOf(book, hrefOf, (i) => partHrefs[i] ?? null);
    const tocWithColophon = [...toc, { label: labels.colophon, href: "text/colophon.xhtml", children: [] }];
    const first = sections[0];
    put(
        "OEBPS/nav.xhtml",
        xmlDocument(
            el(
                "html",
                { xmlns: XHTML_NS, "xmlns:epub": OPS_NS, "xml:lang": lang, lang },
                el("head", {}, el("meta", { charset: "UTF-8" }), el("title", {}, labels.contents), el("link", { rel: "stylesheet", type: "text/css", href: "css/book.css" })),
                el(
                    "body",
                    {},
                    el("nav", { "epub:type": "toc", id: "toc" }, el("h1", {}, labels.contents), navList(tocWithColophon)),
                    el(
                        "nav",
                        { "epub:type": "landmarks", id: "landmarks", hidden: "hidden" },
                        el(
                            "ol",
                            {},
                            cover ? el("li", {}, el("a", { "epub:type": "cover", href: "text/cover.xhtml" }, labels.cover)) : null,
                            el("li", {}, el("a", { "epub:type": "toc", href: "nav.xhtml#toc" }, labels.contents)),
                            first ? el("li", {}, el("a", { "epub:type": "bodymatter", href: hrefOf(first) }, storyLabel(first))) : null,
                        ),
                    ),
                ),
            ),
            "<!DOCTYPE html>",
        ),
    );
    manifest.push({ id: "nav", href: "nav.xhtml", type: "application/xhtml+xml", props: "nav" });

    const counter = { n: 0 };
    put(
        "OEBPS/toc.ncx",
        xmlDocument(
            el(
                "ncx",
                { xmlns: "http://www.daisy.org/z3986/2005/ncx/", version: "2005-1", "xml:lang": lang },
                el("head", {}, el("meta", { name: "dtb:uid", content: book.meta.identifier }), el("meta", { name: "dtb:depth", content: String(Math.max(1, depth(tocWithColophon))) }), el("meta", { name: "dtb:totalPageCount", content: "0" }), el("meta", { name: "dtb:maxPageNumber", content: "0" })),
                el("docTitle", {}, el("text", {}, book.meta.title)),
                el("navMap", {}, ncxPoints(tocWithColophon, counter)),
            ),
        ),
    );
    manifest.push({ id: "ncx", href: "toc.ncx", type: "application/x-dtbncx+xml" });

    put(
        "OEBPS/content.opf",
        xmlDocument(
            el(
                "package",
                { xmlns: "http://www.idpf.org/2007/opf", version: "3.0", "unique-identifier": "uid", "xml:lang": lang },
                el(
                    "metadata",
                    { "xmlns:dc": "http://purl.org/dc/elements/1.1/" },
                    el("dc:identifier", { id: "uid" }, book.meta.identifier),
                    el("dc:title", {}, book.meta.title),
                    el("dc:language", {}, lang),
                    el("dc:creator", {}, "Hypergryph / Yostar (story text and art)"),
                    el("dc:contributor", {}, "myrtle.moe"),
                    book.meta.description ? el("dc:description", {}, book.meta.description) : null,
                    el("meta", { property: "dcterms:modified" }, modified),
                    cover ? el("meta", { name: "cover", content: "cover-image" }) : null,
                ),
                el(
                    "manifest",
                    {},
                    manifest.map((m) => el("item", { id: m.id, href: m.href, "media-type": m.type, properties: m.props })),
                ),
                el(
                    "spine",
                    { toc: "ncx" },
                    spine.map((s) => el("itemref", { idref: s.idref, linear: s.linear === false ? "no" : undefined })),
                ),
            ),
        ),
    );

    zip.end();
    await flush();
    return { blob: sink ? null : new Blob(chunks as BlobPart[], { type: "application/epub+zip" }), images: imageCount, failedImages: failed, files };
}
