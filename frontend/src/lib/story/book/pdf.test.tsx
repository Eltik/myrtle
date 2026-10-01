// @vitest-environment node
import { readFileSync } from "node:fs";
import { PDFArray, PDFDict, PDFDocument, type PDFHexString, PDFName, type PDFRef, type PDFString } from "pdf-lib";
import { describe, expect, it } from "vitest";
import { bookOf } from "./book";
import { say, scriptOf } from "./fixtures";
import { type PdfDeps, toPdf } from "./pdf";
import { sampleSource } from "./sample";
import type { BookOptions, BookScope } from "./types";

// A valid 16x9 PNG (a CG's shape) and the Inter faces the site ships, so fonts and images really embed.
const PNG = Uint8Array.from(Buffer.from("iVBORw0KGgoAAAANSUhEUgAAABAAAAAJCAIAAAC0SDtlAAAAEUlEQVR4nGNoIBEwjGoYFBoAGJ/YAfh07lQAAAAASUVORK5CYII=", "base64"));
const woff = (w: string, s = "normal") => Uint8Array.from(readFileSync(`node_modules/@fontsource/inter/files/inter-latin-${w}-${s}.woff`));

function deps(): PdfDeps & { loads: string[] } {
    const loads: string[] = [];
    return {
        loads,
        loadImage: async (path, _signal, variant) => {
            loads.push(variant === "thumb" ? `thumb ${path}` : path);
            return { bytes: PNG, mime: "image/png" };
        },
        loadFont: async () => ({ regular: woff("400"), bold: woff("700"), italic: woff("400", "italic"), format: "woff" }),
        cover: async () => ({ bytes: PNG, mime: "image/png" }),
    };
}

/** The outline as `title -> p<1-based page>`, indented by depth, read back with pdf-lib. */
async function outline(bytes: Uint8Array): Promise<string[]> {
    const doc = await PDFDocument.load(bytes);
    const pageIndex = new Map(doc.getPages().map((p, i) => [p.ref.toString(), i + 1]));
    const out: string[] = [];
    const walk = (ref: PDFRef | undefined, depth: number) => {
        let at = ref;
        while (at) {
            const node = doc.context.lookup(at, PDFDict);
            const title = (node.lookup(PDFName.of("Title")) as PDFHexString | PDFString).decodeText();
            const dest = node.lookup(PDFName.of("Dest"), PDFArray);
            out.push(`${"  ".repeat(depth)}${title} -> p${pageIndex.get(dest.get(0).toString())}`);
            walk(node.get(PDFName.of("First")) as PDFRef | undefined, depth + 1);
            at = node.get(PDFName.of("Next")) as PDFRef | undefined;
        }
    };
    const root = doc.catalog.lookup(PDFName.of("Outlines"), PDFDict);
    walk(root.get(PDFName.of("First")) as PDFRef | undefined, 0);
    return out;
}

async function build(scope: BookScope, options: Partial<BookOptions> = {}) {
    const book = bookOf(sampleSource, scope, { nickname: "Doctor", images: "cg+bg", branches: "all", ...options });
    const d = deps();
    const result = await toPdf(book, { typeface: "inter", paper: "A5" }, d);
    const bytes = new Uint8Array(await result.blob.arrayBuffer());
    const pages = (await PDFDocument.load(bytes)).getPageCount();
    return { bytes, pages, result, loads: d.loads };
}

describe("toPdf, laid out per story and merged", () => {
    it("a chapter: title, contents, the stories, colophon; the outline nests Before/After under the operation and points at the right pages", async () => {
        const { bytes, pages, result, loads } = await build({ kind: "group", groupId: "main_0" });
        expect(Buffer.from(bytes.subarray(0, 5)).toString()).toBe("%PDF-");
        const [beg, end, int] = result.sections;
        // Title p1, contents p2, then the stories back to back, then the colophon.
        expect(beg.firstPage).toBe(3);
        expect(end.firstPage).toBe(beg.firstPage + beg.pages);
        expect(int.firstPage).toBe(end.firstPage + end.pages);
        expect(pages).toBe(int.firstPage + int.pages);
        expect(await outline(bytes)).toEqual(["Contents -> p2", `0-1 Isolated Island -> p${beg.firstPage}`, `  Before Operation -> p${beg.firstPage}`, `  After Operation -> p${end.firstPage}`, `Prologue <1> -> p${int.firstPage}`, `Colophon -> p${pages}`]);
        expect(loads).toEqual(["thumb /textures/avg/bg/room.png", "/textures/avg/imgs/cg_one.png", "thumb /textures/avg/bg/street.png", "/textures/avg/imgs/cg_two.png"]);
        expect(result.failedImages).toEqual([]);
    }, 30_000);

    it("three books merge in order: a part page each, stories after it, the outline part > story with every target right", async () => {
        const { bytes, pages, result } = await build({ kind: "selection", ids: ["x_1", "s_01_beg", "s_int"] }, { images: "cg" });
        const [x, beg, int] = result.sections;
        // Title, contents, part 1 (p3), X-1 (p4..), part 2, 0-1, the interlude, colophon.
        expect(x.firstPage).toBe(4);
        expect(beg.firstPage).toBe(x.firstPage + x.pages + 1);
        expect(int.firstPage).toBe(beg.firstPage + beg.pages);
        expect(pages).toBe(int.firstPage + int.pages);
        expect(await outline(bytes)).toEqual(["Contents -> p2", "Other & Co -> p3", `  X-1 Elsewhere -> p${x.firstPage}`, `Evil Time Part 1 -> p${beg.firstPage - 1}`, `  0-1 Isolated Island -> p${beg.firstPage}`, `  Prologue <1> -> p${int.firstPage}`, `Colophon -> p${pages}`]);
    }, 30_000);

    it("a story that fails to lay out is named in the error", async () => {
        const book = bookOf(sampleSource, { kind: "group", groupId: "main_0" }, { nickname: "Doctor", images: "none", branches: "all" });
        await expect(toPdf(book, { typeface: "device", paper: "A5", failStoryId: "s_01_end" }, deps())).rejects.toMatchObject({ name: "PdfStoryError", story: "0-1 Isolated Island" });
    }, 30_000);

    it("reports progress per story, pictures then layout, then the merge", async () => {
        const book = bookOf(sampleSource, { kind: "group", groupId: "main_0" }, { nickname: "Doctor", images: "none", branches: "all" });
        const seen: string[] = [];
        await toPdf(book, { typeface: "device", paper: "A5", onProgress: (p) => seen.push(`${p.phase} ${p.done}/${p.total} ${p.label}`) }, deps());
        expect(seen).toEqual(["images 0/3 0-1 Isolated Island", "layout 0/3 0-1 Isolated Island", "images 1/3 0-1 Isolated Island", "layout 1/3 0-1 Isolated Island", "images 2/3 Prologue <1>", "layout 2/3 Prologue <1>", "merge 3/3 "]);
    }, 30_000);

    it("a story that runs over many sheets renders: the page number's line height does not compound per sheet", async () => {
        // Regression: with the line height on <Page>, the fixed page number inherited
        // it and react-pdf multiplied it again on every sheet until pdfkit threw
        // "unsupported number" (main_0 with scenes, sheet 6).
        const long = scriptOf(
            "long",
            Array.from({ length: 160 }, (_, i) => say("Amiya", `Line ${i}: the rain on Chernobog does not stop, and neither do we.`)),
        );
        const book = bookOf({ server: "en", index: { groups: [{ id: "g", name: "G", stories: [{ id: "long", name: "Long", sort: 1, hasScript: true }] }] }, scripts: new Map([["long", long]]) }, { kind: "group", groupId: "g" }, { nickname: "Doctor", images: "none", branches: "all" });
        const out = await toPdf(book, { typeface: "inter", paper: "A5" }, deps());
        expect(out.sections[0].pages).toBeGreaterThan(8);
    }, 30_000);
});
