/**
 * THE BOOK AS A PDF, through `@react-pdf/renderer`, from the same IR the EPUB
 * and the HTML render. One `<Page>` per story (it wraps onto as many sheets as
 * the story needs) with a running head (chapter · story) and a page number; a
 * title page with the composed cover; a contents page whose entries LINK to
 * the stories; a page per part when the book spans several groups; a colophon.
 *
 * BOOKMARKS nest part > operation > story like the EPUB nav. react-pdf nests
 * a bookmark under its nearest bookmarked ANCESTOR, and pages are siblings, so
 * a story page cannot sit under a part page by the tree alone. Its resolver
 * numbers bookmarks breadth-first and then SPREADS the bookmark object over
 * `{ ref, parent }` (`@react-pdf/layout`, `resolveBookmarks`), so a `parent`
 * written on a page's bookmark wins. Page-level bookmarks are numbered first,
 * in page order, so the ref of every earlier page's bookmark is known here and
 * `parent` is set to it. This leans on an internal: `pdf.test.tsx` checks the
 * outline, and a react-pdf upgrade that changes the resolver fails there.
 *
 * Nothing here fetches: images, fonts and the cover come from `PdfDeps`, so
 * the same file renders in Node (tests), on the main thread and in a worker.
 */
import { Document, type DocumentProps, Font, Image, Link, Page, pdf, StyleSheet, Text, View } from "@react-pdf/renderer";
import type React from "react";
import type { TextNode } from "../text";
import { storyLabel } from "./book";
import { type EpubImage, imageKey, imagePaths } from "./epub";
import { type BookLabels, DEFAULT_LABELS } from "./render";
import type { Block, Book, ChoiceBlock, ImageVariant, Section, Typeface } from "./types";

export type PaperSize = "A4" | "LETTER" | "A5";

export interface PdfFont {
    regular: Uint8Array;
    bold: Uint8Array;
    italic?: Uint8Array;
    boldItalic?: Uint8Array;
    format: "woff" | "woff2" | "ttf";
}

export interface PdfDeps {
    loadImage: (path: string, signal?: AbortSignal, variant?: ImageVariant) => Promise<EpubImage | null>;
    loadFont: (typeface: Exclude<Typeface, "device">, signal?: AbortSignal) => Promise<PdfFont | null>;
    cover: (book: Book, signal?: AbortSignal) => Promise<EpubImage | null>;
}

export interface PdfProgress {
    done: number;
    total: number;
    label: string;
}

export interface PdfOptions {
    typeface: Typeface;
    paper: PaperSize;
    labels?: BookLabels;
    signal?: AbortSignal;
    onProgress?: (p: PdfProgress) => void;
    concurrency?: number;
}

export interface PdfResult {
    blob: Blob;
    images: number;
    failedImages: string[];
}

function abortIfNeeded(signal?: AbortSignal): void {
    if (signal?.aborted) throw new DOMException("Export cancelled", "AbortError");
}

function base64(bytes: Uint8Array): string {
    let s = "";
    for (let i = 0; i < bytes.length; i += 0x8000) s += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
    return btoa(s);
}

const FONT_MIME = { woff: "font/woff", woff2: "font/woff2", ttf: "font/ttf" } as const;
let familySeq = 0;

/** Register the face under a fresh family name and return it; the built-in Helvetica for "device". */
function registerFont(font: PdfFont | null): string {
    if (!font) return "Helvetica";
    familySeq += 1;
    const family = `BookFace${familySeq}`;
    const url = (b: Uint8Array) => `data:${FONT_MIME[font.format]};base64,${base64(b)}`;
    Font.register({
        family,
        fonts: [
            { src: url(font.regular), fontWeight: 400 },
            { src: url(font.bold), fontWeight: 700 },
            // A face that ships no italic reads its upright in italic slots rather than failing to resolve.
            { src: url(font.italic ?? font.regular), fontWeight: 400, fontStyle: "italic" },
            { src: url(font.boldItalic ?? font.bold), fontWeight: 700, fontStyle: "italic" },
        ],
    });
    return family;
}

// Words are never split: react-pdf's default English hyphenation breaks names mid-word.
Font.registerHyphenationCallback((word) => [word]);

const SCALE: Record<PaperSize, number> = { A5: 10, A4: 11, LETTER: 11 };
/**
 * Line height of running text. react-pdf scales this by the FACE's own line
 * metrics, not the em: with Inter at 10 pt, 1.0 set lines 1.8 em apart and
 * 1.45 set them 2.6 em apart (measured off the rendered pages). 0.85 lands
 * near 1.5 em, the EPUB's `line-height`.
 */
const LH = 0.85;
/** Page widths in points, and the side margin, so a scene thumbnail gets an absolute width. */
const PAGE_WIDTH: Record<PaperSize, number> = { A5: 419.53, A4: 595.28, LETTER: 612 };
const SIDE: Record<PaperSize, number> = { A5: 42, A4: 60, LETTER: 60 };

function stylesFor(paper: PaperSize, family: string) {
    const fs = SCALE[paper];
    return StyleSheet.create({
        // NO lineHeight here: a fixed element on a wrapping page inherits it and
        // react-pdf multiplies it again on every sheet (the page number reached a
        // line height of 1.09e9 pt by sheet 6 and pdfkit threw "unsupported
        // number"). Inherited from a View it compounds too (a line of dialogue
        // took 3.3 lines), so each text style carries its own.
        page: { fontFamily: family, fontSize: fs, paddingTop: 54, paddingBottom: 54, paddingHorizontal: SIDE[paper], color: "#111" },
        body: {},
        head: { position: "absolute", top: 22, left: 0, right: 0, textAlign: "center", fontSize: fs * 0.72, color: "#777" },
        foot: { position: "absolute", bottom: 22, left: 0, right: 0, textAlign: "center", fontSize: fs * 0.75, color: "#777" },
        eyebrow: { fontSize: fs * 0.75, letterSpacing: 1, color: "#666", textTransform: "uppercase" },
        title: { fontSize: fs * 1.8, fontWeight: 700, marginTop: 2, marginBottom: 8 },
        synopsis: { lineHeight: LH, fontStyle: "italic", color: "#444", marginBottom: 14 },
        para: { lineHeight: LH, marginBottom: fs * 0.5 },
        who: { fontWeight: 700 },
        narr: { lineHeight: LH, fontStyle: "italic", color: "#333", marginBottom: fs * 0.5 },
        overlay: { lineHeight: LH, textAlign: "center", marginVertical: fs * 0.8 },
        scene: { marginTop: fs * 1.2, marginBottom: fs, alignItems: "center" },
        // A third of the text column, in points, at the backgrounds' 16:9.
        thumb: { width: (PAGE_WIDTH[paper] - 2 * SIDE[paper]) / 3, height: ((PAGE_WIDTH[paper] - 2 * SIDE[paper]) / 3) * (9 / 16), objectFit: "contain", marginBottom: fs * 0.6 },
        rule: { width: "30%", borderTopWidth: 0.6, borderTopColor: "#bbb" },
        figure: { marginVertical: fs, alignItems: "center" },
        cg: { width: "100%" },
        choice: { marginVertical: fs, paddingLeft: fs * 0.8, borderLeftWidth: 2, borderLeftColor: "#222" },
        recall: { borderLeftColor: "#999" },
        label: { fontSize: fs * 0.72, letterSpacing: 0.8, color: "#666", textTransform: "uppercase", marginBottom: 3 },
        option: { lineHeight: LH, marginBottom: 2 },
        arm: { marginTop: fs * 0.7 },
        cutscene: { textAlign: "center", fontSize: fs * 0.75, letterSpacing: 1, color: "#777", marginVertical: fs },
        center: { alignItems: "center", justifyContent: "center", height: "100%" },
        bigTitle: { fontSize: fs * 2.2, fontWeight: 700, textAlign: "center", marginTop: 16 },
        tocTitle: { fontSize: fs * 1.6, fontWeight: 700, marginBottom: 12 },
        tocPart: { fontWeight: 700, marginTop: 8, marginBottom: 3 },
        tocItem: { marginBottom: 3, color: "#111", textDecoration: "none" },
        small: { fontSize: fs * 0.85, color: "#444", marginBottom: 6 },
    });
}

type Styles = ReturnType<typeof stylesFor>;

interface Ctx {
    s: Styles;
    labels: BookLabels;
    book: Book;
    src: (path: string, variant?: ImageVariant) => Blob | null;
}

function inline(nodes: readonly TextNode[], key = "n"): React.ReactNode[] {
    return nodes.map((n, i) => {
        const k = `${key}.${i}`;
        if (n.kind === "text") return n.value;
        if (n.kind === "inline")
            return (
                <Text key={k} style={n.tag === "i" ? { fontStyle: "italic" } : n.tag === "b" ? { fontWeight: 700 } : { textDecoration: "underline" }}>
                    {inline(n.children, k)}
                </Text>
            );
        if (n.kind === "color")
            return (
                <Text key={k} style={/^#[0-9a-fA-F]{3,8}$|^[a-zA-Z]+$/.test(n.color) ? { color: n.color } : {}}>
                    {inline(n.children, k)}
                </Text>
            );
        return (
            <Text key={k}>
                {inline(n.children, k)}
                {"\n"}
            </Text>
        );
    });
}

function plain(nodes: readonly TextNode[]): string {
    return nodes.map((n) => (n.kind === "text" ? n.value : plain(n.children))).join("");
}

function choice(b: ChoiceBlock, ctx: Ctx): React.ReactNode {
    const { s } = ctx;
    const optionText = (values: string[]) => b.options.filter((o) => values.includes(o.value)).map((o) => plain(o.nodes));
    if (b.chosen !== undefined) {
        const picked = b.options.find((o) => o.value === b.chosen);
        return (
            <View key={b.id}>
                {b.asked && picked ? (
                    <Text style={s.para}>
                        <Text style={s.who}>{ctx.book.meta.options.nickname}</Text> {inline(picked.nodes)}
                    </Text>
                ) : null}
                {b.arms.flatMap((arm) => blocks(arm.blocks, ctx))}
            </View>
        );
    }
    return (
        <View key={b.id} style={b.asked ? s.choice : [s.choice, s.recall]}>
            {b.asked ? (
                <View wrap={false}>
                    <Text style={s.label}>{ctx.labels.choice}</Text>
                    {b.options.map((o, i) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: options repeat values (main_15-06 has three `1`s); the position is their identity.
                        <Text key={i} style={s.option}>
                            {`${i + 1}. `}
                            {inline(o.nodes)}
                        </Text>
                    ))}
                </View>
            ) : null}
            {b.arms.map((arm, i) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: two arms can name the same values; the position is their identity.
                <View key={i} style={s.arm}>
                    <Text style={s.label}>{b.asked ? ctx.labels.ifChose(optionText(arm.values)) : ctx.labels.earlier(optionText(arm.values))}</Text>
                    {blocks(arm.blocks, ctx)}
                </View>
            ))}
        </View>
    );
}

function block(b: Block, ctx: Ctx): React.ReactNode {
    const { s } = ctx;
    const images = ctx.book.meta.options.images;
    switch (b.kind) {
        case "line":
            return (
                <Text key={b.id} style={s.para}>
                    <Text style={s.who}>{b.speaker}</Text> {inline(b.nodes)}
                </Text>
            );
        case "narration":
            return (
                <Text key={b.id} style={s.narr}>
                    {inline(b.nodes)}
                </Text>
            );
        case "overlay":
            return (
                <Text key={b.id} style={s.overlay}>
                    {inline(b.nodes)}
                </Text>
            );
        case "scene": {
            const src = images === "cg+bg" && b.url ? ctx.src(b.url, "thumb") : null;
            return (
                <View key={b.id} style={s.scene} wrap={false}>
                    {src ? <Image src={src} style={s.thumb} /> : null}
                    <View style={s.rule} />
                </View>
            );
        }
        case "figure": {
            if (images === "none") return null;
            const srcs = b.images.flatMap((i) => {
                const src = ctx.src(i.url);
                return src ? [src] : [];
            });
            if (srcs.length === 0) return null;
            return (
                <View key={b.id} style={s.figure} wrap={false}>
                    {srcs.map((src, i) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: panels of one figure have no other identity.
                        <Image key={i} src={src} style={s.cg} />
                    ))}
                </View>
            );
        }
        case "choice":
            return choice(b, ctx);
        case "cutscene":
            return (
                <Text key={b.id} style={s.cutscene}>
                    {ctx.labels.cutscene.toUpperCase()}
                </Text>
            );
    }
}

function blocks(list: readonly Block[], ctx: Ctx): React.ReactNode[] {
    return list.map((b) => block(b, ctx));
}

/** A react-pdf bookmark with an explicit parent ref (see the file head). */
function mark(title: string, parent?: number): { title: string; parent?: number } {
    return parent === undefined ? { title } : { title, parent };
}

/** The document tree. Exported for the test, which renders it without deps. */
export function pdfDocument(book: Book, opts: { paper: PaperSize; family: string; labels: BookLabels; cover: Blob | null; src: Ctx["src"] }): React.ReactElement<DocumentProps> {
    const s = stylesFor(opts.paper, opts.family);
    const ctx: Ctx = { s, labels: opts.labels, book, src: opts.src };
    const multi = book.parts.length > 1;
    const pages: React.ReactElement[] = [];
    let ref = 0; // the next page-level bookmark's ref
    const sectionId = (sec: Section) => `story-${sec.id.replace(/[^A-Za-z0-9_-]/g, "_")}`;

    pages.push(
        <Page key="title" size={opts.paper} style={s.page}>
            <View style={s.center}>
                {opts.cover ? <Image src={opts.cover} style={{ width: "100%", objectFit: "contain", maxHeight: "80%" }} /> : null}
                <Text style={s.bigTitle}>{book.meta.title}</Text>
            </View>
        </Page>,
    );
    const contentsRef = ref++;
    pages.push(
        <Page key="toc" size={opts.paper} style={s.page} bookmark={mark(opts.labels.contents) as never}>
            <Text style={s.tocTitle}>{opts.labels.contents}</Text>
            {book.parts.map((part) => (
                <View key={part.id}>
                    {multi ? <Text style={s.tocPart}>{part.title}</Text> : null}
                    {part.sections.map((sec) => (
                        <Link key={sec.id} src={`#${sectionId(sec)}`} style={s.tocItem}>
                            {`${multi ? "   " : ""}${storyLabel(sec)}${sec.tag ? ` · ${sec.tag}` : ""}`}
                        </Link>
                    ))}
                </View>
            ))}
        </Page>,
    );
    void contentsRef;

    for (const part of book.parts) {
        let partRef: number | undefined;
        if (multi) {
            partRef = ref++;
            const art = book.meta.partArt?.[part.id];
            const artSrc = art ? opts.src(art) : null;
            pages.push(
                <Page key={`part-${part.id}`} size={opts.paper} style={s.page} bookmark={mark(part.title) as never}>
                    <View style={s.center}>
                        {artSrc ? <Image src={artSrc} style={{ width: "100%", objectFit: "contain", maxHeight: "70%" }} /> : null}
                        <Text style={s.bigTitle}>{part.title}</Text>
                    </View>
                </Page>,
            );
        }
        const list = part.sections;
        for (let i = 0; i < list.length; i += 1) {
            const code = list[i].code?.trim();
            let j = i;
            while (code && j + 1 < list.length && list[j + 1].code?.trim() === code) j += 1;
            const group = list.slice(i, j + 1);
            const grouped = group.length > 1;
            const opRef = ref;
            group.forEach((sec, gi) => {
                const pageMark = grouped ? (gi === 0 ? mark(storyLabel(sec), partRef) : mark(sec.tag || sec.name, opRef)) : mark(storyLabel(sec), partRef);
                ref += 1;
                const eyebrow = [sec.code, sec.tag].filter(Boolean).join(" · ");
                pages.push(
                    <Page key={sec.id} size={opts.paper} style={s.page} wrap bookmark={pageMark as never}>
                        <Text style={s.head} fixed>
                            {`${part.title} · ${storyLabel(sec)}`}
                        </Text>
                        <View style={s.body}>
                            <View id={sectionId(sec)} {...(grouped && gi === 0 ? ({ bookmark: sec.tag || sec.name } as object) : {})}>
                                {eyebrow ? <Text style={s.eyebrow}>{eyebrow}</Text> : null}
                                <Text style={s.title}>{sec.name}</Text>
                                {sec.synopsis ? <Text style={s.synopsis}>{sec.synopsis}</Text> : null}
                            </View>
                            {blocks(sec.blocks, ctx)}
                        </View>
                        <Text style={s.foot} fixed render={({ pageNumber }) => String(pageNumber)} />
                    </Page>,
                );
            });
            i = j;
        }
    }

    pages.push(
        <Page key="colophon" size={opts.paper} style={s.page} bookmark={mark(opts.labels.colophon) as never}>
            <Text style={s.tocTitle}>{book.meta.title}</Text>
            <Text style={s.small}>{opts.labels.credit}</Text>
            <Text style={s.small}>{opts.labels.rights}</Text>
            <Text style={s.small}>{opts.labels.madeWith}</Text>
            <Text style={s.small}>{book.meta.identifier}</Text>
        </Page>,
    );

    return (
        <Document title={book.meta.title} author="Hypergryph / Yostar (story text and art)" creator="myrtle.moe" producer="myrtle.moe" language={book.meta.language}>
            {pages}
        </Document>
    );
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

export async function toPdf(book: Book, opts: PdfOptions, deps: PdfDeps): Promise<PdfResult> {
    const { signal } = opts;
    const labels = opts.labels ?? DEFAULT_LABELS;
    const sections = book.parts.flatMap((p) => p.sections);
    const font = opts.typeface === "device" ? null : await deps.loadFont(opts.typeface, signal);
    const family = registerFont(font);
    const coverImage = await deps.cover(book, signal);
    const blobs = new Map<string, Blob>();
    const failed: string[] = [];
    const load = async (key: string) => {
        abortIfNeeded(signal);
        const variant: ImageVariant = key.startsWith("thumb:") ? "thumb" : "full";
        const path = variant === "thumb" ? key.slice("thumb:".length) : key;
        let image: EpubImage | null = null;
        try {
            image = await deps.loadImage(path, signal, variant);
        } catch (err) {
            if (signal?.aborted) throw err;
        }
        if (image) blobs.set(key, new Blob([image.bytes as BlobPart], { type: image.mime }));
        else failed.push(key);
    };
    // Part pages carry each group's key visual.
    const partArt = Object.values(book.meta.partArt ?? {}).filter((p): p is string => Boolean(p));
    if (book.parts.length > 1) await mapLimit([...new Set(partArt)], opts.concurrency ?? 4, load);
    let done = 0;
    for (const section of sections) {
        abortIfNeeded(signal);
        opts.onProgress?.({ done, total: sections.length, label: storyLabel(section) });
        const wanted = [...new Set(imagePaths(section.blocks, book.meta.options.images))].filter((k) => !blobs.has(k) && !failed.includes(k));
        await mapLimit(wanted, opts.concurrency ?? 4, load);
        done += 1;
    }
    opts.onProgress?.({ done, total: sections.length, label: "" });
    abortIfNeeded(signal);
    const doc = pdfDocument(book, {
        paper: opts.paper,
        family,
        labels,
        cover: coverImage ? new Blob([coverImage.bytes as BlobPart], { type: coverImage.mime }) : null,
        src: (path, variant) => blobs.get(imageKey(path, variant)) ?? null,
    });
    const blob = await pdf(doc).toBlob();
    abortIfNeeded(signal);
    return { blob, images: blobs.size, failedImages: failed };
}
