import { describe, expect, it } from "vitest";
import { bookOf } from "./book";
import { bookCss } from "./css";
import { toHtml } from "./html";
import { toMarkdown, toText } from "./plain";
import { sampleSource } from "./sample";
import type { BookOptions } from "./types";
import { el, escapeAttr, escapeText, serialize } from "./xml";

const OPTIONS: BookOptions = { nickname: "Kal", images: "cg+bg", branches: "all" };
const book = () => bookOf(sampleSource, { kind: "group", groupId: "main_0" }, OPTIONS);

describe("the XML builder", () => {
    it("escapes text and attributes, drops characters XML forbids, and never parses a string as markup", () => {
        expect(escapeText(`a < b & c > "d"`)).toBe(`a &lt; b &amp; c &gt; "d"`);
        expect(escapeAttr(`x"y\nz`)).toBe("x&quot;y&#10;z");
        expect(serialize(el("p", { title: `"<>"` }, "<script>alert(1)</script>", "\u0007bell\ud800"))).toBe(`<p title="&quot;&lt;&gt;&quot;">&lt;script&gt;alert(1)&lt;/script&gt;bell</p>`);
    });

    it("self-closes void elements only, so the output parses the same as HTML and XML", () => {
        expect(serialize(el("div", {}, el("br"), el("span", {})))).toBe("<div><br/><span></span></div>");
        expect(() => serialize(el("bad name"))).toThrow();
    });
});

describe("toHtml", () => {
    it("is one well-formed document with unique ids, a toc, the stylesheet and a class per colour", () => {
        const html = toHtml(book(), { typeface: "inter", imageSrc: (p) => `https://cdn${p}` });
        const doc = new DOMParser().parseFromString(html, "text/html");
        const ids = [...doc.querySelectorAll("[id]")].map((e) => e.id);
        expect(new Set(ids).size).toBe(ids.length);
        expect([...doc.querySelectorAll("nav a")].map((a) => a.getAttribute("href"))).toEqual(["#s1-top", "#s2-top", "#s3-top"]);
        for (const a of doc.querySelectorAll("nav a")) expect(doc.getElementById(a.getAttribute("href")?.slice(1) ?? "")).not.toBeNull();
        expect(doc.querySelector(".tc-ff6600")?.textContent).toBe("burning");
        expect(doc.querySelector("style")?.textContent).toContain(".tc-ff6600{color:#ff6600}");
        expect([...doc.querySelectorAll("img")].map((i) => i.getAttribute("src"))).toEqual([
            "https://cdn/textures/avg/bg/room.png",
            "https://cdn/textures/avg/imgs/cg_one.png",
            "https://cdn/textures/avg/bg/street.png",
            "https://cdn/textures/avg/bg/street.png",
            "https://cdn/textures/avg/imgs/cg_one.png",
            "https://cdn/textures/avg/imgs/cg_two.png",
        ]);
        // The option with "&" and "<here>" arrives as text, not markup.
        expect([...doc.querySelectorAll(".option")].map((o) => o.textContent)).toEqual(["Run.", "Stay & fight <here>."]);
        expect(doc.querySelector(".sticker")?.innerHTML).toBe("Chernobog<br>11:47 A.M.");
        // A scene is an uncaptioned thumbnail over a rule: the file key is printed nowhere.
        const scene = doc.querySelector(".scene");
        expect(scene?.id).toBe("s1-b1");
        expect(scene?.innerHTML).toBe('<figure class="scene-thumb"><img src="https://cdn/textures/avg/bg/room.png" alt=""></figure><hr>');
        expect(html).not.toContain("bg_room");
        expect(html).not.toMatch(/>\s*room\s*</);
    });
});

describe("stylesheet", () => {
    it("embeds the chosen face, and a device face names no file", () => {
        expect(bookCss("opendyslexic", { regular: "../fonts/r.woff2", bold: "../fonts/b.woff2", format: "woff2" }, [])).toContain(`src:url("../fonts/r.woff2") format("woff2")`);
        expect(bookCss("device", null, [])).not.toContain("@font-face");
        expect(bookCss("inter", null, ["#abc", "red", "url(x)"])).toMatch(/\.tc-abc\{color:#abc\}\n\.tc-red\{color:red\}\n$/);
    });
});

describe("Markdown and text", () => {
    it("Markdown", () => {
        expect(toMarkdown(book(), { imageUrl: (p) => `https://cdn${p}` })).toMatchSnapshot();
    });

    it("scenes are a bare rule in Markdown and text, with or without backgrounds", () => {
        const md = toMarkdown(book(), { imageUrl: (p) => `https://cdn${p}` });
        expect(md).not.toContain("room.png");
        expect(md).not.toContain("street.png");
        expect(md).toContain("![CG](https://cdn/textures/avg/imgs/cg_one.png)");
    });

    it("text", () => {
        expect(toText(book(), { imageUrl: (p) => `https://cdn${p}` })).toMatchSnapshot();
    });

    it("my path prints the Doctor's pick as their line and the branch as ordinary lines", () => {
        const path = bookOf(sampleSource, { kind: "story", storyId: "s_01_beg" }, { ...OPTIONS, images: "none", branches: "path", choices: { s_01_beg: { 0: "2" } } });
        const text = toText(path);
        expect(text).toContain("Kal: Stay & fight <here>.\n\n---\n\nAmiya: Then we fight.");
        expect(text).not.toContain("This way!");
        expect(text).not.toContain("[CG]");
    });
});
