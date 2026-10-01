/**
 * THE BOOK AS A PDF, through `@react-pdf/renderer`, from the same IR the EPUB
 * and the HTML render: a title page with the composed cover, a contents page
 * WITH PAGE NUMBERS, a page per part when the book spans several groups, each
 * story on its own sheets with a running head (chapter · story) and a page
 * number, a colophon, and an outline part > operation > story.
 *
 * LAID OUT ONE STORY AT A TIME, THEN MERGED. react-pdf lays a document out in
 * one synchronous pass and keeps the whole tree alive until it is written:
 * over the SHADOW OF A DYING SUN arc (225 stories) that pass sat on "Laying
 * out pages" for 60 s with no progress while the worker heap climbed from 20
 * MB to 2,198 MB (register, 2026-10-01), and a worker that crosses the heap
 * limit takes the whole tab down with it. So every story, every part page,
 * the title page and the colophon are separate react-pdf documents, each
 * copied into one `pdf-lib` document as soon as it is written and then
 * dropped. The running head is given the story's page OFFSET, so the numbers
 * run on across documents; the contents is laid out twice, once with blank
 * numbers to learn its length (which fixes every later page number) and once
 * at the end with the real ones, and slotted in after the title page. The
 * outline is written by hand (`writeOutline`): pdf-lib copies pages, not
 * bookmarks.
 *
 * Nothing here fetches: images, fonts and the cover come from `PdfDeps`, so
 * the same file renders in Node (tests), on the main thread and in a worker.
 */
import { Document, type DocumentProps, Font, Image, Page, pdf, StyleSheet, Text, View } from "@react-pdf/renderer";
import { type PDFArray, type PDFDict, PDFDocument, PDFHexString, PDFName, PDFNull, PDFNumber, type PDFRef } from "pdf-lib";
import type React from "react";
import type { TextNode } from "../text";
import { storyLabel } from "./book";
import { type EpubImage, imageKey, imagePaths } from "./epub";
import { PdfStoryError } from "./pdfError";
import { type BookLabels, DEFAULT_LABELS } from "./render";
import type { Block, Book, ChoiceBlock, ImageVariant, Section, Typeface } from "./types";

export type PaperSize = "A4" | "LETTER" | "A5";
export { PdfStoryError };

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
    /** `images`: a story's pictures are loading; `layout`: the story is being laid out; `merge`: the file is being written. */
    phase: "images" | "layout" | "merge";
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
    /** Test hook (`?pdffail=<story id>`): laying out this story throws, to prove a failure reaches the sheet. */
    failStoryId?: string;
}

export interface PdfResult {
    blob: Blob;
    images: number;
    failedImages: string[];
    /** Where each story landed, 1-based, for the tests and the register. */
    sections: { id: string; firstPage: number; pages: number }[];
    pages: number;
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
        tocItem: { marginBottom: 3, color: "#111" },
        tocRow: { flexDirection: "row", alignItems: "flex-end" },
        tocLabel: { flexGrow: 1, flexShrink: 1, paddingRight: 8 },
        tocNumber: { width: 32, textAlign: "right" },
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

/** One entry of the outline, with the 0-based page it opens. */
export interface OutlineNode {
    title: string;
    page: number;
    children: OutlineNode[];
}

function countOf(nodes: readonly OutlineNode[]): number {
    return nodes.reduce((n, node) => n + 1 + countOf(node.children), 0);
}

/**
 * Write `/Outlines` into a pdf-lib document: one dictionary per entry with
 * Title, Parent, Prev/Next, First/Last, Count and a `/XYZ` destination on its
 * page. Every entry is open, so Count is its descendants.
 */
export function writeOutline(doc: PDFDocument, nodes: readonly OutlineNode[]): void {
    const ctx = doc.context;
    const pages = doc.getPages();
    const rootRef = ctx.nextRef();
    const build = (list: readonly OutlineNode[], parent: PDFRef): { first?: PDFRef; last?: PDFRef } => {
        const refs = list.map(() => ctx.nextRef());
        list.forEach((node, i) => {
            const dict = ctx.obj({}) as PDFDict;
            dict.set(PDFName.of("Title"), PDFHexString.fromText(node.title));
            dict.set(PDFName.of("Parent"), parent);
            const page = pages[Math.min(Math.max(node.page, 0), pages.length - 1)];
            dict.set(PDFName.of("Dest"), ctx.obj([page.ref, PDFName.of("XYZ"), PDFNull, PDFNull, PDFNull]) as PDFArray);
            if (i > 0) dict.set(PDFName.of("Prev"), refs[i - 1]);
            if (i < list.length - 1) dict.set(PDFName.of("Next"), refs[i + 1]);
            if (node.children.length > 0) {
                const kids = build(node.children, refs[i]);
                if (kids.first) dict.set(PDFName.of("First"), kids.first);
                if (kids.last) dict.set(PDFName.of("Last"), kids.last);
                dict.set(PDFName.of("Count"), PDFNumber.of(countOf(node.children)));
            }
            ctx.assign(refs[i], dict);
        });
        return { first: refs[0], last: refs[refs.length - 1] };
    };
    const root = ctx.obj({}) as PDFDict;
    root.set(PDFName.of("Type"), PDFName.of("Outlines"));
    const top = build(nodes, rootRef);
    if (top.first) root.set(PDFName.of("First"), top.first);
    if (top.last) root.set(PDFName.of("Last"), top.last);
    root.set(PDFName.of("Count"), PDFNumber.of(countOf(nodes)));
    ctx.assign(rootRef, root);
    doc.catalog.set(PDFName.of("Outlines"), rootRef);
    doc.catalog.set(PDFName.of("PageMode"), PDFName.of("UseOutlines"));
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

async function bytesOf(doc: React.ReactElement<DocumentProps>): Promise<Uint8Array> {
    const blob = await pdf(doc).toBlob();
    return new Uint8Array(await blob.arrayBuffer());
}

/** Yield to the event loop, so progress messages and a cancel get through between stories. */
const tick = () => new Promise<void>((resolve) => setTimeout(resolve, 0));

/** Consecutive stories sharing an operation code, as the EPUB nav groups them. */
function operations(sections: readonly Section[]): Section[][] {
    const out: Section[][] = [];
    for (let i = 0; i < sections.length; i += 1) {
        const code = sections[i].code?.trim();
        let j = i;
        while (code && j + 1 < sections.length && sections[j + 1].code?.trim() === code) j += 1;
        out.push(sections.slice(i, j + 1));
        i = j;
    }
    return out;
}

export async function toPdf(book: Book, opts: PdfOptions, deps: PdfDeps): Promise<PdfResult> {
    const { signal } = opts;
    const labels = opts.labels ?? DEFAULT_LABELS;
    const sections = book.parts.flatMap((p) => p.sections);
    const multi = book.parts.length > 1;
    const font = opts.typeface === "device" ? null : await deps.loadFont(opts.typeface, signal);
    /**
     * A FRESH font family for every document. react-pdf keeps a parsed face in
     * its font store, and a face reused by a second document broke that
     * document's line breaking (a word's first letter left on the line above:
     * "fan t / he flames", 9-5 Critical Value) and its text layer ("Than- you"
     * for "Thank you"); one document never showed it. `Font.reset()` does not
     * help (the next render reads a null face: "reading 'unitsPerEm'"), so each
     * document registers the same bytes under a new name and the old family is
     * dropped from the store once its document is written.
     */
    let family = registerFont(font);
    let s = stylesFor(opts.paper, family);
    const fresh = () => {
        const fonts = Font.getRegisteredFonts() as Record<string, unknown>;
        if (family !== "Helvetica") delete fonts[family];
        family = registerFont(font);
        s = stylesFor(opts.paper, family);
        ctx.s = s;
        return s;
    };
    const coverImage = await deps.cover(book, signal);
    const doc = (children: React.ReactNode) => (
        <Document title={book.meta.title} author="Hypergryph / Yostar (story text and art)" creator="myrtle.moe" producer="myrtle.moe" language={book.meta.language}>
            {children}
        </Document>
    );

    // Images, each distinct file once per book, loaded just before the first story that shows it.
    const blobs = new Map<string, Blob>();
    const failed: string[] = [];
    const load = async (key: string) => {
        abortIfNeeded(signal);
        if (blobs.has(key) || failed.includes(key)) return;
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
    const ctx: Ctx = { s, labels, book, src: (path, variant) => blobs.get(imageKey(path, variant)) ?? null };
    /** Build a document against a fresh face, then write it. */
    const render = (build: () => React.ReactElement<DocumentProps>) => {
        fresh();
        return bytesOf(build());
    };

    const merged = await PDFDocument.create();
    const append = async (bytes: Uint8Array): Promise<number> => {
        const src = await PDFDocument.load(bytes);
        const copied = await merged.copyPages(src, src.getPageIndices());
        for (const page of copied) merged.addPage(page);
        return copied.length;
    };

    // The contents, with numbers once they are known and blanks before.
    const contents = (numbers: Map<string, number> | null) =>
        doc(
            <Page size={opts.paper} style={s.page} wrap>
                <Text style={s.tocTitle}>{labels.contents}</Text>
                {book.parts.map((part) => (
                    <View key={part.id}>
                        {multi ? (
                            <View style={s.tocRow} wrap={false}>
                                <Text style={[s.tocPart, s.tocLabel]}>{part.title}</Text>
                                <Text style={[s.tocPart, s.tocNumber]}>{numbers ? String(numbers.get(`part:${part.id}`) ?? "") : ""}</Text>
                            </View>
                        ) : null}
                        {part.sections.map((sec) => (
                            <View key={sec.id} style={s.tocRow} wrap={false}>
                                <Text style={[s.tocItem, s.tocLabel]}>{`${multi ? "   " : ""}${storyLabel(sec)}${sec.tag ? ` · ${sec.tag}` : ""}`}</Text>
                                <Text style={[s.tocItem, s.tocNumber]}>{numbers ? String(numbers.get(sec.id) ?? "") : ""}</Text>
                            </View>
                        ))}
                    </View>
                ))}
                <View style={s.tocRow} wrap={false}>
                    <Text style={[s.tocItem, s.tocLabel]}>{labels.colophon}</Text>
                    <Text style={[s.tocItem, s.tocNumber]}>{numbers ? String(numbers.get("colophon") ?? "") : ""}</Text>
                </View>
            </Page>,
        );

    await append(
        await render(() =>
            doc(
                <Page size={opts.paper} style={s.page}>
                    <View style={s.center}>
                        {coverImage ? <Image src={new Blob([coverImage.bytes as BlobPart], { type: coverImage.mime })} style={{ width: "100%", objectFit: "contain", maxHeight: "80%" }} /> : null}
                        <Text style={s.bigTitle}>{book.meta.title}</Text>
                    </View>
                </Page>,
            ),
        ),
    );
    // 2. The contents' LENGTH, which every later page number depends on.
    const contentsPages = (await PDFDocument.load(await render(() => contents(null)))).getPageCount();
    let next = 1 + contentsPages + 1; // the 1-based number of the next page to be written
    const numbers = new Map<string, number>();
    const placed: PdfResult["sections"] = [];
    const outline: OutlineNode[] = [{ title: labels.contents, page: 1, children: [] }];

    let done = 0;
    for (const part of book.parts) {
        let into = outline;
        if (multi) {
            abortIfNeeded(signal);
            const art = book.meta.partArt?.[part.id];
            if (art && book.meta.options.images !== "none") await load(art);
            const artSrc = art ? blobs.get(art) : null;
            numbers.set(`part:${part.id}`, next);
            const n = await append(
                await render(() =>
                    doc(
                        <Page size={opts.paper} style={s.page}>
                            <View style={s.center}>
                                {artSrc ? <Image src={artSrc} style={{ width: "100%", objectFit: "contain", maxHeight: "70%" }} /> : null}
                                <Text style={s.bigTitle}>{part.title}</Text>
                            </View>
                        </Page>,
                    ),
                ),
            );
            const node: OutlineNode = { title: part.title, page: next - 1, children: [] };
            outline.push(node);
            into = node.children;
            next += n;
        }
        for (const group of operations(part.sections)) {
            const grouped = group.length > 1;
            const opNode: OutlineNode | null = grouped ? { title: storyLabel(group[0]), page: next - 1, children: [] } : null;
            if (opNode) into.push(opNode);
            for (const sec of group) {
                abortIfNeeded(signal);
                opts.onProgress?.({ phase: "images", done, total: sections.length, label: storyLabel(sec) });
                const wanted = [...new Set(imagePaths(sec.blocks, book.meta.options.images))];
                await mapLimit(wanted, opts.concurrency ?? 4, load);
                abortIfNeeded(signal);
                opts.onProgress?.({ phase: "layout", done, total: sections.length, label: storyLabel(sec) });
                await tick();
                const offset = next - 1;
                const eyebrow = [sec.code, sec.tag].filter(Boolean).join(" · ");
                let n: number;
                try {
                    if (opts.failStoryId === sec.id) throw new Error("synthetic failure (?pdffail)");
                    n = await append(
                        await render(() =>
                            doc(
                                <Page size={opts.paper} style={s.page} wrap>
                                    <Text style={s.head} fixed>
                                        {`${part.title} · ${storyLabel(sec)}`}
                                    </Text>
                                    <View>
                                        {eyebrow ? <Text style={s.eyebrow}>{eyebrow}</Text> : null}
                                        <Text style={s.title}>{sec.name}</Text>
                                        {sec.synopsis ? <Text style={s.synopsis}>{sec.synopsis}</Text> : null}
                                    </View>
                                    {blocks(sec.blocks, ctx)}
                                    <Text style={s.foot} fixed render={({ pageNumber }) => String(offset + pageNumber)} />
                                </Page>,
                            ),
                        ),
                    );
                } catch (err) {
                    if (signal?.aborted) throw err;
                    throw new PdfStoryError(storyLabel(sec), err);
                }
                numbers.set(sec.id, next);
                placed.push({ id: sec.id, firstPage: next, pages: n });
                const title = grouped ? sec.tag || sec.name : storyLabel(sec);
                (opNode ? opNode.children : into).push({ title, page: next - 1, children: [] });
                next += n;
                done += 1;
            }
        }
    }

    numbers.set("colophon", next);
    outline.push({ title: labels.colophon, page: next - 1, children: [] });
    await append(
        await render(() =>
            doc(
                <Page size={opts.paper} style={s.page}>
                    <Text style={s.tocTitle}>{book.meta.title}</Text>
                    <Text style={s.small}>{labels.credit}</Text>
                    <Text style={s.small}>{labels.rights}</Text>
                    <Text style={s.small}>{labels.madeWith}</Text>
                    <Text style={s.small}>{book.meta.identifier}</Text>
                </Page>,
            ),
        ),
    );

    opts.onProgress?.({ phase: "merge", done, total: sections.length, label: "" });
    abortIfNeeded(signal);
    const final = await PDFDocument.load(await render(() => contents(numbers)));
    if (final.getPageCount() !== contentsPages) throw new Error(`contents changed length (${contentsPages} -> ${final.getPageCount()} pages)`);
    const tocPages = await merged.copyPages(final, final.getPageIndices());
    tocPages.forEach((page, i) => {
        merged.insertPage(1 + i, page);
    });
    writeOutline(merged, outline);
    merged.setTitle(book.meta.title);
    merged.setAuthor("Hypergryph / Yostar (story text and art)");
    merged.setCreator("myrtle.moe");
    merged.setProducer("myrtle.moe");
    merged.setLanguage(book.meta.language);
    const out = await merged.save();
    return { blob: new Blob([out as BlobPart], { type: "application/pdf" }), images: blobs.size, failedImages: failed, sections: placed, pages: merged.getPageCount() };
}
