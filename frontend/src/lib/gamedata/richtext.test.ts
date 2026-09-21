import { describe, expect, it } from "vitest";

import { emphasizeTagsHtml, foldRichText, parseRichText, plainText, renderTags } from "./richtext";

const wrap = (tag: string, content: string) => `<span data-tag="${tag}">${content}</span>`;

describe("parseRichText", () => {
    it("reads a Unity inline tag as a span, not as text", () => {
        expect(parseRichText("A <i>clear</i> battle plan.")).toEqual([
            { kind: "text", value: "A " },
            { kind: "inline", tag: "i", children: [{ kind: "text", value: "clear" }] },
            { kind: "text", value: " battle plan." },
        ]);
    });

    it("nests a game span inside a game span and closes inside out", () => {
        expect(parseRichText("<$cc.x><@cc.kw>k</> tail</>")).toEqual([
            {
                kind: "style",
                tag: "$cc.x",
                children: [
                    { kind: "style", tag: "@cc.kw", children: [{ kind: "text", value: "k" }] },
                    { kind: "text", value: " tail" },
                ],
            },
        ]);
    });

    it("keeps an angle-bracketed item name as text", () => {
        expect(parseRichText("<@lv.item><Roadblock></>")).toEqual([{ kind: "style", tag: "@lv.item", children: [{ kind: "text", value: "<Roadblock>" }] }]);
    });

    it("drops the Unity tags the browser drops and keeps their text", () => {
        expect(parseRichText("<color name=#ffffff>LIMITED</color> <size=20>x</size>")).toEqual([
            { kind: "text", value: "LIMITED" },
            { kind: "text", value: " " },
            { kind: "text", value: "x" },
        ]);
    });

    it("degrades malformed input to its text", () => {
        expect(plainText("</i>stray</> and <i>open")).toBe("stray and open");
        // A `</>` arriving while an inline span is open closes the inline span too.
        expect(renderTags("<@ba.vup><i>x</>y", wrap)).toBe('<span data-tag="@ba.vup"><i>x</i></span>y');
    });
});

describe("renderTags", () => {
    it("escapes literal text and emits inline tags as HTML", () => {
        expect(renderTags("a & b <I>c</I>", wrap)).toBe("a &amp; b <i>c</i>");
    });

    it("emphasizes game spans and turns newlines into breaks", () => {
        expect(emphasizeTagsHtml("<@lv.rem>r</>\nnext")).toBe('<strong style="color: var(--foreground)">r</strong><br/>next');
    });
});

describe("foldRichText", () => {
    it("visits text leaves in reading order and folds each branch over its children", () => {
        const seen: string[] = [];
        const out = foldRichText(parseRichText("a<i>b<@x.kw>c</>d</i>e"), {
            text: (v) => {
                seen.push(v);
                return v;
            },
            inline: (tag, children) => `${tag}(${children.join("")})`,
            style: (tag, children) => `${tag}[${children.join("")}]`,
        });
        expect(seen).toEqual(["a", "b", "c", "d", "e"]);
        expect(out.join("")).toBe("ai(b@x.kw[c]d)e");
    });
});

describe("plainText", () => {
    it("strips every tag so a search matches across them", () => {
        expect(plainText("A <i>clear</i> battle <@ba.kw>plan</>.")).toBe("A clear battle plan.");
    });
});
