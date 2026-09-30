// @vitest-environment node
import { readFileSync } from "node:fs";
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

/** Outline titles in document order with their depth, read off the PDF's outline dictionaries. */
function outline(bytes: Uint8Array): string[] {
    const text = Buffer.from(bytes).toString("latin1");
    const objects = new Map<number, string>();
    for (const m of text.matchAll(/(\d+) 0 obj\s*([\s\S]*?)endobj/g)) objects.set(Number(m[1]), m[2]);
    const decode = (raw: string): string => {
        const hex = /^<([0-9a-fA-F]+)>$/.exec(raw);
        if (hex) {
            const b = Buffer.from(hex[1], "hex");
            return b[0] === 0xfe && b[1] === 0xff ? b.subarray(2).swap16().toString("utf16le") : b.toString("latin1");
        }
        return raw.slice(1, -1);
    };
    const ref = (body: string, key: string) => {
        const m = new RegExp(`/${key} (\\d+) 0 R`).exec(body);
        return m ? Number(m[1]) : null;
    };
    const catalog = [...objects.values()].find((b) => /\/Type \/Catalog/.test(b)) ?? "";
    const rootId = ref(catalog, "Outlines");
    const root = rootId === null ? undefined : objects.get(rootId);
    const out: string[] = [];
    const walk = (id: number | null, depth: number) => {
        while (id !== null) {
            const body = objects.get(id) ?? "";
            const title = /\/Title (\([^)]*\)|<[0-9a-fA-F]+>)/.exec(body);
            out.push(`${"  ".repeat(depth)}${title ? decode(title[1]) : "?"}`);
            walk(ref(body, "First"), depth + 1);
            id = ref(body, "Next");
        }
    };
    if (root) walk(ref(root, "First"), 0);
    return out;
}

async function build(scope: BookScope, options: Partial<BookOptions> = {}) {
    const book = bookOf(sampleSource, scope, { nickname: "Doctor", images: "cg+bg", branches: "all", ...options });
    const d = deps();
    const result = await toPdf(book, { typeface: "inter", paper: "A5" }, d);
    const bytes = new Uint8Array(await result.blob.arrayBuffer());
    const pages = (
        Buffer.from(bytes)
            .toString("latin1")
            .match(/\/Type \/Page\b/g) ?? []
    ).length;
    return { bytes, pages, result, loads: d.loads };
}

describe("toPdf", () => {
    it("a chapter: title, contents, one page per story, colophon; the outline nests Before/After under the operation", async () => {
        const { bytes, pages, result, loads } = await build({ kind: "group", groupId: "main_0" });
        expect(Buffer.from(bytes.subarray(0, 5)).toString()).toBe("%PDF-");
        // Title, contents, the two 0-1 stories on four A5 sheets between them (CGs, scene thumbnails), the interlude, colophon.
        expect(pages).toBe(8);
        expect(outline(bytes)).toEqual(["Contents", "0-1 Isolated Island", "  Before Operation", "  After Operation", "Prologue <1>", "Colophon"]);
        expect(loads).toEqual(["thumb /textures/avg/bg/room.png", "/textures/avg/imgs/cg_one.png", "thumb /textures/avg/bg/street.png", "/textures/avg/imgs/cg_two.png"]);
        expect(result.failedImages).toEqual([]);
        expect(Buffer.from(bytes).toString("latin1")).toMatch(/\/BaseFont \/[A-Z]{6}\+Inter-Regular/);
    }, 30_000);

    it("a selection across groups adds a part page per group and nests part > story", async () => {
        const { pages, bytes } = await build({ kind: "selection", ids: ["x_1", "s_int"] }, { images: "cg" });
        expect(pages).toBe(7);
        expect(outline(bytes)).toEqual(["Contents", "Other & Co", "  X-1 Elsewhere", "Evil Time Part 1", "  Prologue <1>", "Colophon"]);
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
        const pages = (
            Buffer.from(await out.blob.arrayBuffer())
                .toString("latin1")
                .match(/\/Type \/Page\b/g) ?? []
        ).length;
        expect(pages).toBeGreaterThan(8);
    }, 30_000);
});
