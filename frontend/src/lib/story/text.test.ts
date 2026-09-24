import { describe, expect, it } from "vitest";
import { parseStoryText, plainStoryText, renderLine, substitute } from "./text";

describe("substitute", () => {
    it("replaces the nickname in both spellings, the nbs token, and literal newlines", () => {
        expect(substitute("Hello, {@nickname}. {@Nickname}!", "Amiya")).toBe("Hello, Amiya. Amiya!");
        expect(substitute("Ave{@nbs}Mujica", "x")).toBe("Ave Mujica");
        expect(substitute("a\\nb", "x")).toBe("a\nb");
    });
});

describe("parseStoryText", () => {
    it("keeps plain text as one node", () => {
        expect(parseStoryText("Just text.")).toEqual([{ kind: "text", value: "Just text." }]);
    });

    it("nests <i> and closes on </i>", () => {
        expect(parseStoryText("a <i>b</i> c")).toEqual([
            { kind: "text", value: "a " },
            { kind: "inline", tag: "i", children: [{ kind: "text", value: "b" }] },
            { kind: "text", value: " c" },
        ]);
    });

    it("reads <color=#hex> with </color> and with the universal </> closer", () => {
        expect(parseStoryText("<color=#d41f1f>red</color> and <color=#888888>grey</>")).toEqual([
            { kind: "color", color: "#d41f1f", children: [{ kind: "text", value: "red" }] },
            { kind: "text", value: " and " },
            { kind: "color", color: "#888888", children: [{ kind: "text", value: "grey" }] },
        ]);
    });

    it("splits <p=N> paragraphs", () => {
        const nodes = parseStoryText("<p=2>September 28th</>, <p=1>Rhodes Island</>");
        expect(nodes).toEqual([
            { kind: "paragraph", index: 2, children: [{ kind: "text", value: "September 28th" }] },
            { kind: "text", value: ", " },
            { kind: "paragraph", index: 1, children: [{ kind: "text", value: "Rhodes Island" }] },
        ]);
        expect(plainStoryText(nodes)).toBe("September 28th\n, Rhodes Island\n");
    });

    it("colours <@tu.kw> like the rest of the site and leaves unknown brackets as text", () => {
        const nodes = parseStoryText("<@tu.kw>Orundum</> in <The Romance of Ives>");
        expect(nodes[0]).toMatchObject({ kind: "color", color: "#27e8e7" });
        expect(nodes[1]).toEqual({ kind: "text", value: " in <The Romance of Ives>" });
    });

    it("never throws on unbalanced markup", () => {
        expect(() => parseStoryText("</i></></color><i>open")).not.toThrow();
        expect(plainStoryText(parseStoryText("</i></></color><i>open"))).toBe("open");
    });

    it("renderLine substitutes then parses", () => {
        expect(plainStoryText(renderLine("<i>{@nickname}</i>", "Doctor"))).toBe("Doctor");
    });
});
