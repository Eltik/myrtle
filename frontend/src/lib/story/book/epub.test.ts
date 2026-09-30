import { strFromU8, unzipSync } from "fflate";
import { describe, expect, it } from "vitest";
import { bookOf } from "./book";
import { type EpubDeps, type EpubProgress, toEpub } from "./epub";
import { sampleSource } from "./sample";
import type { BookOptions, BookScope } from "./types";

const JPEG = new Uint8Array([0xff, 0xd8, 0xff, 0xd9]);
const PNG = new Uint8Array([0x89, 0x50, 0x4e, 0x47]);

function deps(overrides: Partial<EpubDeps> = {}): EpubDeps & { loads: string[] } {
    const loads: string[] = [];
    return {
        loads,
        loadImage: async (path, _signal, variant) => {
            loads.push(variant === "thumb" ? `thumb ${path}` : path);
            if (path.includes("street")) return { bytes: PNG, mime: "image/png" };
            return { bytes: JPEG, mime: "image/jpeg" };
        },
        loadFont: async () => ({ regular: new Uint8Array([1]), bold: new Uint8Array([2]), format: "woff" }),
        cover: async () => ({ bytes: JPEG, mime: "image/jpeg" }),
        ...overrides,
    };
}

async function build(scope: BookScope, options: Partial<BookOptions> = {}, d = deps()) {
    const book = bookOf(sampleSource, scope, { nickname: "Doctor", images: "cg+bg", branches: "all", ...options });
    const progress: EpubProgress[] = [];
    const result = await toEpub(book, { typeface: "inter", modified: new Date("2026-09-30T12:34:56.789Z"), onProgress: (p) => progress.push(p) }, d);
    const bytes = new Uint8Array(await (result.blob as Blob).arrayBuffer());
    return { book, result, bytes, entries: unzipSync(bytes), progress, deps: d };
}

const parse = (xml: string, type: DOMParserSupportedType = "application/xhtml+xml") => {
    const doc = new DOMParser().parseFromString(xml, type);
    expect(doc.getElementsByTagName("parsererror").length, xml.slice(0, 200)).toBe(0);
    return doc;
};

/** Resolve `href` against the directory of `from`, both inside the zip. */
function resolve(from: string, href: string): string {
    const parts = from.split("/").slice(0, -1);
    for (const seg of href.split("#")[0].split("/")) {
        if (seg === "..") parts.pop();
        else if (seg !== "." && seg !== "") parts.push(seg);
    }
    return parts.join("/");
}

describe("toEpub", () => {
    it("writes mimetype FIRST and STORED, with no extra field", async () => {
        const { bytes } = await build({ kind: "group", groupId: "main_0" });
        const view = new DataView(bytes.buffer, bytes.byteOffset);
        expect(view.getUint32(0, true)).toBe(0x04034b50);
        expect(view.getUint16(8, true)).toBe(0); // compression: stored
        expect(view.getUint16(26, true)).toBe(8); // name length
        expect(view.getUint16(28, true)).toBe(0); // extra length
        expect(strFromU8(bytes.slice(30, 38))).toBe("mimetype");
        expect(strFromU8(bytes.slice(38, 58))).toBe("application/epub+zip");
    });

    it("container -> OPF; the manifest lists exactly the files in the zip; the spine is cover, nav, the stories, colophon", async () => {
        const { entries, book } = await build({ kind: "group", groupId: "main_0" });
        const container = parse(strFromU8(entries["META-INF/container.xml"]), "application/xml");
        const opfPath = container.getElementsByTagName("rootfile")[0].getAttribute("full-path") ?? "";
        expect(opfPath).toBe("OEBPS/content.opf");
        const opf = parse(strFromU8(entries[opfPath]), "application/xml");
        const items = [...opf.getElementsByTagName("item")];
        const hrefs = items.map((i) => resolve(opfPath, i.getAttribute("href") ?? "")).sort();
        const files = Object.keys(entries)
            .filter((f) => f !== "mimetype" && f !== "META-INF/container.xml" && f !== opfPath)
            .sort();
        expect(hrefs).toEqual(files);
        const byId = new Map(items.map((i) => [i.getAttribute("id"), i.getAttribute("href")]));
        const spine = [...opf.getElementsByTagName("itemref")].map((r) => byId.get(r.getAttribute("idref")));
        const sections = book.parts.flatMap((p) => p.sections);
        expect(spine).toEqual(["text/cover.xhtml", "nav.xhtml", ...sections.map((s) => `text/${s.id}.xhtml`), "text/colophon.xhtml"]);
        const meta = (tag: string) => opf.getElementsByTagName(tag)[0]?.textContent;
        expect(meta("dc:identifier")).toBe("urn:myrtle:story:en:main_0");
        expect(meta("dc:creator")).toBe("Hypergryph / Yostar (story text and art)");
        expect(meta("dc:contributor")).toBe("myrtle.moe");
        expect(meta("dc:language")).toBe("en");
        expect(meta("dc:description")).toContain("Amiya wakes the Doctor.");
        expect(opf.querySelector('meta[property="dcterms:modified"]')?.textContent).toBe("2026-09-30T12:34:56Z");
        expect(items.find((i) => i.getAttribute("properties") === "cover-image")?.getAttribute("href")).toBe("images/cover.jpg");
        expect(items.find((i) => i.getAttribute("properties") === "nav")?.getAttribute("href")).toBe("nav.xhtml");
    });

    it("the nav nests an operation's Before and After under it, carries landmarks, and the NCX agrees", async () => {
        const { entries } = await build({ kind: "group", groupId: "main_0" });
        const nav = parse(strFromU8(entries["OEBPS/nav.xhtml"]));
        const toc = nav.querySelector("nav#toc > ol") as Element;
        const tree = (ol: Element): unknown[] => [...ol.children].map((li) => [li.querySelector(":scope > a")?.textContent, li.querySelector(":scope > a")?.getAttribute("href"), ...(li.querySelector(":scope > ol") ? [tree(li.querySelector(":scope > ol") as Element)] : [])]);
        expect(tree(toc)).toEqual([
            [
                "0-1 Isolated Island",
                "text/s_01_beg.xhtml",
                [
                    ["Before Operation", "text/s_01_beg.xhtml"],
                    ["After Operation", "text/s_01_end.xhtml"],
                ],
            ],
            ["Prologue <1>", "text/s_int.xhtml"],
            ["Colophon", "text/colophon.xhtml"],
        ]);
        expect([...nav.querySelectorAll("nav#landmarks a")].map((a) => a.getAttributeNS("http://www.idpf.org/2007/ops", "type"))).toEqual(["cover", "toc", "bodymatter"]);
        const ncx = parse(strFromU8(entries["OEBPS/toc.ncx"]), "application/xml");
        expect(ncx.getElementsByTagName("navPoint").length).toBe(5);
        expect(ncx.querySelector('meta[name="dtb:uid"]')?.getAttribute("content")).toBe("urn:myrtle:story:en:main_0");
    });

    it("a selection across groups nests part > story, with a page per part", async () => {
        const { entries } = await build({ kind: "selection", ids: ["x_1", "s_int"] });
        const nav = parse(strFromU8(entries["OEBPS/nav.xhtml"]));
        const top = [...(nav.querySelector("nav#toc > ol") as Element).children].map((li) => [li.querySelector(":scope > a")?.textContent, li.querySelectorAll("li").length]);
        expect(top).toEqual([
            ["Other & Co", 1],
            ["Evil Time Part 1", 1],
            ["Colophon", 0],
        ]);
        expect(Object.keys(entries)).toContain("OEBPS/text/part-1.xhtml");
    });

    it("every document is well-formed XHTML and every src and href inside the book resolves", async () => {
        const { entries } = await build({ kind: "selection", ids: ["s_01_beg", "s_01_end", "s_int", "x_1"] });
        for (const [path, data] of Object.entries(entries)) {
            if (!path.endsWith(".xhtml")) continue;
            const doc = parse(strFromU8(data));
            for (const node of doc.querySelectorAll("[src], [href]")) {
                const ref = node.getAttribute("src") ?? node.getAttribute("href") ?? "";
                expect(entries[resolve(path, ref)], `${path} -> ${ref}`).toBeDefined();
            }
        }
        const css = strFromU8(entries["OEBPS/css/book.css"]);
        expect(css).toContain('src:url("../fonts/inter-400.woff") format("woff")');
        expect(entries["OEBPS/fonts/inter-400.woff"]).toBeDefined();
        expect(css).toContain(".tc-ff6600{color:#ff6600}");
    });

    it("loads each image once per book, keeps PNG where the loader says so, and reports progress per story", async () => {
        const { result, progress, deps: d, entries } = await build({ kind: "group", groupId: "main_0" });
        expect(d.loads).toEqual(["thumb /textures/avg/bg/room.png", "/textures/avg/imgs/cg_one.png", "thumb /textures/avg/bg/street.png", "/textures/avg/imgs/cg_two.png"]);
        expect(result.images).toBe(4);
        expect(Object.keys(entries).filter((f) => f.startsWith("OEBPS/images/i"))).toEqual(["OEBPS/images/i1.jpg", "OEBPS/images/i2.jpg", "OEBPS/images/i3.png", "OEBPS/images/i4.jpg"]);
        expect(progress.map((p) => `${p.done}/${p.total} ${p.label}`)).toEqual(["0/3 0-1 Isolated Island", "1/3 0-1 Isolated Island", "2/3 Prologue <1>", "3/3 "]);
    });

    it("CGs only leaves backgrounds out; none leaves every picture out; a failed image is reported, not fatal", async () => {
        const cg = await build({ kind: "group", groupId: "main_0" }, { images: "cg" });
        expect(cg.deps.loads).toEqual(["/textures/avg/imgs/cg_one.png", "/textures/avg/imgs/cg_two.png"]);
        const none = await build({ kind: "group", groupId: "main_0" }, { images: "none" });
        expect(none.deps.loads).toEqual([]);
        const failing = await build({ kind: "group", groupId: "main_0" }, { images: "cg" }, deps({ loadImage: async (p) => (p.includes("two") ? null : { bytes: JPEG, mime: "image/jpeg" }) }));
        expect(failing.result.failedImages).toEqual(["/textures/avg/imgs/cg_two.png"]);
        expect(strFromU8(failing.entries["OEBPS/text/s_01_end.xhtml"])).not.toContain("cg_two");
    });

    it("stops on abort", async () => {
        const controller = new AbortController();
        const book = bookOf(sampleSource, { kind: "group", groupId: "main_0" }, { nickname: "Doctor", images: "cg", branches: "all" });
        const run = toEpub(book, { typeface: "device", signal: controller.signal, onProgress: (p) => p.done === 1 && controller.abort() }, deps());
        await expect(run).rejects.toThrow();
    });

    it("streams into a sink as it goes: mimetype first, every entry in order, writes before the book ends, no Blob", async () => {
        const book = bookOf(sampleSource, { kind: "selection", ids: ["x_1", "s_01_beg", "s_int"] }, { nickname: "Doctor", images: "cg+bg", branches: "all" });
        const writes: Uint8Array[] = [];
        const progressAtWrite: number[] = [];
        let done = 0;
        const sink = {
            write: async (chunk: Uint8Array) => {
                // A slow disk: the writer must wait for it rather than pile chunks up.
                await new Promise((r) => setTimeout(r, 1));
                writes.push(chunk);
                progressAtWrite.push(done);
            },
        };
        const result = await toEpub(book, { typeface: "inter", modified: new Date("2026-09-30T00:00:00Z"), sink, onProgress: (p) => (done = p.done) }, deps());
        expect(result.blob).toBeNull();
        expect(progressAtWrite.some((d) => d < 3)).toBe(true);
        const bytes = new Uint8Array(writes.reduce((n, w) => n + w.length, 0));
        let at = 0;
        for (const w of writes) {
            bytes.set(w, at);
            at += w.length;
        }
        expect(strFromU8(bytes.slice(30, 38))).toBe("mimetype");
        const entries = unzipSync(bytes);
        expect(Object.keys(entries)).toEqual(result.files);
        expect(result.files.slice(0, 2)).toEqual(["mimetype", "META-INF/container.xml"]);
        // A part page per group, carrying the group's key visual when it has one.
        expect(strFromU8(entries["OEBPS/text/part-2.xhtml"])).toMatch(/<figure class="part-art"><img src="\.\.\/images\/i\d+\.jpg" alt=""\/><\/figure><h1>Evil Time Part 1<\/h1>/);
        expect(strFromU8(entries["OEBPS/text/part-1.xhtml"])).toMatch(/<figure class="part-art">/);
    });
});
