import { describe, expect, it } from "vitest";
import { plainStoryText } from "../text";
import { type Cmd, decide, gate, say, scriptOf } from "./fixtures";
import type { Block, BookOptions, ChoiceBlock } from "./types";
import { sectionOf } from "./walk";

const ALL: BookOptions = { nickname: "Doctor", images: "cg+bg", branches: "all" };

function walk(commands: Cmd[], options: BookOptions = ALL, assets = {}) {
    return sectionOf(scriptOf("s", commands, assets), null, options);
}

/** A compact picture of a block tree: `Amiya: hi`, `> A|B`, `[1] ...`. */
function shape(blocks: readonly Block[]): unknown[] {
    return blocks.map((b): unknown => {
        switch (b.kind) {
            case "line":
                return `${b.speaker}: ${plainStoryText(b.nodes)}`;
            case "narration":
                return `~ ${plainStoryText(b.nodes)}`;
            case "overlay":
                return `${b.style}: ${plainStoryText(b.nodes)}`;
            case "scene":
                return `# ${b.name}`;
            case "figure":
                return `[${b.source} ${b.images.map((i) => i.name).join("/")}]`;
            case "cutscene":
                return `(video ${b.res})`;
            case "choice":
                return {
                    [`${b.asked ? ">" : "recall"} ${b.options.map((o) => `${o.value}=${plainStoryText(o.nodes)}`).join("|")}${b.chosen !== undefined ? ` chosen ${b.chosen}` : ""}`]: b.arms.map((a) => ({ [a.values.join(";")]: shape(a.blocks) })),
                };
            default:
                return null;
        }
    });
}

describe("decisions", () => {
    it("a one-option decision joined at once is the Doctor's reply, and the lines after it are everyone's", () => {
        const s = walk([say("Amiya", "Ready?"), decide(["Let's go."], ["1"]), gate("1"), say("Amiya", "Then go.")]);
        expect(shape(s.blocks)).toEqual(["Amiya: Ready?", { "> 1=Let's go.": [] }, "Amiya: Then go."]);
    });

    it("two, three and four options each keep EVERY option and its gated lines, then the shared continuation", () => {
        for (const n of [2, 3, 4]) {
            const values = Array.from({ length: n }, (_, i) => String(i + 1));
            const cmds: Cmd[] = [
                decide(
                    values.map((v) => `opt${v}`),
                    values,
                ),
            ];
            for (const v of values) cmds.push(gate(v), say("A", `reply ${v}`));
            cmds.push(gate(...values), say("A", "after"));
            const s = walk(cmds);
            const choice = s.blocks[0] as ChoiceBlock;
            expect(choice.options.map((o) => o.value)).toEqual(values);
            expect(choice.arms.map((a) => a.values)).toEqual(values.map((v) => [v]));
            expect(choice.arms.map((a) => shape(a.blocks))).toEqual(values.map((v) => [`A: reply ${v}`]));
            expect(shape(s.blocks.slice(1))).toEqual(["A: after"]);
        }
    });

    it("a gate naming several options is one arm those options share", () => {
        const s = walk([decide(["a", "b", "c"], ["1", "2", "3"]), gate("1", "2"), say("A", "a or b"), gate("3"), say("A", "c"), gate("1", "2", "3"), say("A", "all")]);
        expect(shape(s.blocks)).toEqual([{ "> 1=a|2=b|3=c": [{ "1;2": ["A: a or b"] }, { "3": ["A: c"] }] }, "A: all"]);
    });

    it("nests a decision inside a branch when values are numbered across decisions (act12d0_01)", () => {
        const s = walk([decide(["outer a", "outer b"], ["1", "2"]), gate("1"), say("A", "in a"), decide(["inner x", "inner y"], ["3", "4"]), gate("3"), say("A", "x"), gate("4"), say("A", "y"), gate("2"), say("A", "in b"), gate("3", "4", "2"), say("A", "joined")]);
        expect(shape(s.blocks)).toEqual([
            {
                "> 1=outer a|2=outer b": [{ "1": ["A: in a", { "> 3=inner x|4=inner y": [{ "3": ["A: x"] }, { "4": ["A: y"] }] }] }, { "2": ["A: in b"] }],
            },
            "A: joined",
        ]);
    });

    it("a gate that joins an inner decision and names the outer option CONTINUES that option's arm (main_15-06)", () => {
        const s = walk([decide(["yes", "no"], ["1", "2"]), gate("1"), decide(["p", "q", "r"], ["1", "1", "1"]), gate("1"), say("A", "after inner"), gate("2"), say("A", "said no"), gate("1", "2"), say("A", "end")]);
        expect(shape(s.blocks)).toEqual([{ "> 1=yes|2=no": [{ "1": [{ "> 1=p|1=q|1=r": [] }, "A: after inner"] }, { "2": ["A: said no"] }] }, "A: end"]);
    });

    it("a decision with no gate after it closes at the next decision, and an EMPTY gate joins everything", () => {
        const s = walk([decide(["a", "b"], ["1", "2"]), decide(["c", "d"], ["1", "2"]), gate("1"), say("A", "c"), gate(), say("A", "Sounds good.")]);
        expect(shape(s.blocks)).toEqual([{ "> 1=a|2=b": [] }, { "> 1=c|2=d": [{ "1": ["A: c"] }] }, "A: Sounds good."]);
    });

    it("a gate on the LATEST decision after its join is a recall in place; one on an older decision is counted unreachable", () => {
        const s = walk([decide(["a", "b"], ["1", "2"]), gate("1", "2"), say("A", "shared"), gate("2"), say("A", "only b"), gate("1", "2"), say("A", "end"), decide(["c"], ["3"]), gate("3"), gate("1"), say("A", "dead")]);
        expect(shape(s.blocks)).toEqual([{ "> 1=a|2=b": [] }, "A: shared", { "recall 1=a|2=b": [{ "2": ["A: only b"] }] }, "A: end", { "> 3=c": [] }]);
        expect(s.stats.dropped["gate:unreachable:line"]).toBe(1);
    });

    it("an option text with a literal ';' past the last value stays one option (main_08-14_end)", () => {
        const s = walk([["decision", { options: "a;b;c; d", values: "1;2;3" }]]);
        expect((s.blocks[0] as ChoiceBlock).options.map((o) => plainStoryText(o.nodes))).toEqual(["a", "b", "c; d"]);
    });

    it("more values than option texts shows the options there are (main_06-02: one option, values 1;2;3)", () => {
        const s = walk([["decision", { options: "The usual, or...?", values: "1;2;3" }], gate("1"), say("Blaze", "Fastest!")]);
        expect(shape(s.blocks)).toEqual([{ "> 1=The usual, or...?": [] }, "Blaze: Fastest!"]);
    });

    it("a decision with no values is dropped and counted", () => {
        const s = walk([["decision", { options: "a" }], say("A", "x")]);
        expect(shape(s.blocks)).toEqual(["A: x"]);
        expect(s.stats.dropped["decision:novalues"]).toBe(1);
    });
});

describe("the reader's path", () => {
    const cmds: Cmd[] = [decide(["a", "b"], ["1", "2"]), gate("1"), say("A", "took a"), decide(["x", "y"], ["3", "4"]), gate("3"), say("A", "x"), gate("4"), say("A", "y"), gate("2"), say("A", "took b"), gate("1", "2", "3", "4"), say("A", "end")];

    it("follows the saved values by ENGINE ordinal: decisions numbered in the order the path reaches them", () => {
        const s = walk(cmds, { ...ALL, branches: "path", choices: { s: { 0: "1", 1: "4" } } });
        expect(shape(s.blocks)).toEqual([{ "> 1=a|2=b chosen 1": [{ "1": ["A: took a", { "> 3=x|4=y chosen 4": [{ "4": ["A: y"] }] }] }] }, "A: end"]);
    });

    it("takes the first option where nothing is saved or the saved value is not an option", () => {
        const s = walk(cmds, { ...ALL, branches: "path", choices: { s: { 0: "9" } } });
        expect(shape(s.blocks)).toEqual([{ "> 1=a|2=b chosen 1": [{ "1": ["A: took a", { "> 3=x|4=y chosen 3": [{ "3": ["A: x"] }] }] }] }, "A: end"]);
        expect(s.stats.words).toBe(["a", "took", "a", "x", "x", "end"].length);
    });

    it("keeps a recall only when its decision went the recalled way", () => {
        const c: Cmd[] = [decide(["a", "b"], ["1", "2"]), gate("1", "2"), gate("2"), say("A", "only b"), gate("1", "2")];
        expect(shape(walk(c, { ...ALL, branches: "path", choices: { s: { 0: "1" } } }).blocks)).toEqual([{ "> 1=a|2=b chosen 1": [] }]);
        expect(shape(walk(c, { ...ALL, branches: "path", choices: { s: { 0: "2" } } }).blocks)).toEqual([{ "> 1=a|2=b chosen 2": [] }, { "recall 1=a|2=b chosen 2": [{ "2": ["A: only b"] }] }]);
    });
});

describe("text", () => {
    it("substitutes {@nickname} and {@nbs}, keeps colour spans and italics as nodes", () => {
        const s = walk([say("{@nickname}", "Hi, {@nickname}.{@nbs}<color=#ff6600>Hot</color> <i>now</i>")], { ...ALL, nickname: "Kal" });
        const line = s.blocks[0] as Extract<Block, { kind: "line" }>;
        expect(line.speaker).toBe("Kal");
        expect(line.nodes).toEqual([
            { kind: "text", value: "Hi, Kal. " },
            { kind: "color", color: "#ff6600", children: [{ kind: "text", value: "Hot" }] },
            { kind: "text", value: " " },
            { kind: "inline", tag: "i", children: [{ kind: "text", value: "now" }] },
        ]);
    });

    it("an empty speaker is narration, and an empty line is counted, not printed", () => {
        const s = walk([say("", "A voice."), say("A", "  "), ["text", {}, "Rain."], ["dialog", {}]]);
        expect(shape(s.blocks)).toEqual(["~ A voice.", "~ Rain."]);
        expect(s.stats.dropped["text:empty"]).toBe(1);
    });

    it("stickers keep their newlines; a repeated id without multi HIDES (the engine's rule), with multi it prints", () => {
        const s = walk([
            ["sticker", { id: "st1", text: "Line one\\nLine two" }],
            ["sticker", { id: "st1", text: "hidden" }],
            ["sticker", { id: "st2", text: "a" }],
            ["sticker", { id: "st2", multi: "true", text: "b" }],
            ["subtitle", { text: "Far\\naway" }],
            ["subtitle", { text: "off", x: "-5" }],
        ]);
        expect(shape(s.blocks)).toEqual(["sticker: Line one\nLine two", "sticker: a", "sticker: b", "subtitle: Far\naway"]);
        expect(s.stats.dropped).toMatchObject({ "sticker:repeatHides": 1, "subtitle:offcanvas": 1 });
    });
});

describe("scenes, pictures and cutscenes", () => {
    const assets = {
        backgrounds: { bg_a: "/bg/a.png", bg_b: "/bg/b.png", bg_c: "/bg/c.png", p1: "/bg/p1.png", p2: "/bg/p2.png" },
        images: { cg1: "/cg/1.png", item: "/items/i.png" },
        videos: { "video/v.mp4": { mp4Url: "/v.mp4" } },
    };

    it("a scene on every background CHANGE, a figure per CG, a panel strip as one figure", () => {
        const s = walk(
            [
                ["background", { image: "bg_a" }],
                ["background", { image: "bg_a", screenadapt: "coverall" }],
                ["image", { image: "cg1" }],
                ["image", { image: "cg1", fadetime: "1" }],
                ["image", {}],
                ["image", { image: "missing" }],
                ["cgitem", { image: "item" }],
                ["showitem", { image: "item" }],
                ["largebg", { imagegroup: "p1/p2" }],
                ["gridbg", { imagegroup: "p1/nope" }],
            ],
            ALL,
            assets,
        );
        expect(shape(s.blocks)).toEqual(["# bg_a", "[image cg1]", "[cgitem item]", "[showitem item]", "[largebg p1/p2]"]);
        expect(s.stats.dropped).toEqual({ "background:same": 1, "image:same": 1, "image:unresolved": 1, "gridbg:unresolved": 1 });
    });

    it("a background change in one branch is still a change in the next, and the scene after the join prints again", () => {
        const s = walk([["background", { image: "bg_a" }], decide(["a", "b"], ["1", "2"]), gate("1"), ["background", { image: "bg_b" }], say("A", "in b"), gate("2"), ["background", { image: "bg_b" }], gate("1", "2"), ["background", { image: "bg_b" }]], ALL, assets);
        expect(shape(s.blocks)).toEqual(["# bg_a", { "> 1=a|2=b": [{ "1": ["# bg_b", "A: in b"] }, { "2": ["# bg_b"] }] }, "# bg_b"]);
    });

    it("a video that resolves is a cutscene; one EN retired is counted", () => {
        const s = walk([["video", { res: "video/v.mp4" }], ["video", { res: "video/gone.mp4" }], say("A", "after")], ALL, assets);
        expect(shape(s.blocks)).toEqual(["(video video/v.mp4)", "A: after"]);
        expect(s.stats.dropped["video:unresolved"]).toBe(1);
    });
});
