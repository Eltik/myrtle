/**
 * THE WHOLE EN CORPUS through the book walk: nothing dropped silently, every
 * decision's every option on the page. Reads the scripts pulled to a local
 * folder (`GET /story/{id}` per story, one JSON each); set STORY_CORPUS_DIR,
 * or the test skips.
 */
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import type { StoryScript } from "#/types/generated/StoryScript";
import { plainStoryText, renderLine } from "../text";
import { toText } from "./plain";
import type { Block, SectionStats } from "./types";
import { sectionOf } from "./walk";

const DIR = process.env.STORY_CORPUS_DIR ?? "/private/tmp/claude-501/-Users-eltik-Documents-Coding-myrtle/e756f466-3684-428a-a841-00e255473b3e/scratchpad/bookcensus/st";
// Present AND non-empty: a swept scratch folder is an empty directory, not a corpus.
const present = existsSync(DIR) && readdirSync(DIR).some((f) => f.endsWith(".json"));

const n = (bag: Record<string, number | undefined>, ...keys: string[]) => keys.reduce((t, k) => t + (bag[k] ?? 0), 0);

/** The per-kind ledger: commands in == blocks out + counted drops. */
function ledger(s: SectionStats) {
    const { in: i, out: o, dropped: d } = s;
    const u = (k: string) => d[`gate:unreachable:${k}`] ?? 0;
    return {
        lines: [n(i, "name", "multiline", "text", "narration", "dialog", "voicewithin") - n(i, "dialog:bare", "voicewithin:bare"), n(o, "line", "narration") + n(d, "text:empty") + u("line") + u("narration")],
        overlays: [n(i, "subtitle", "sticker") - n(i, "subtitle:bare", "sticker:bare"), n(o, "overlay") + n(d, "subtitle:offcanvas", "sticker:repeatHides") + u("overlay")],
        scenes: [n(i, "background") - n(i, "background:bare"), n(o, "scene") + n(d, "background:same") + u("scene")],
        figures: [n(i, "image", "cgitem", "showitem", "largebg", "gridbg") - n(i, "image:bare", "cgitem:bare", "showitem:bare", "largebg:bare", "gridbg:bare"), n(o, "figure") + n(d, "image:same", "image:unresolved", "cgitem:unresolved", "showitem:unresolved", "largebg:unresolved", "gridbg:unresolved") + u("figure")],
        choices: [n(i, "decision"), n(o, "choice") + n(d, "decision:novalues") + u("choice")],
        cutscenes: [n(i, "video"), n(o, "cutscene") + n(d, "video:unresolved") + u("cutscene")],
    };
}

function choicesIn(blocks: readonly Block[], out: Extract<Block, { kind: "choice" }>[] = []) {
    for (const b of blocks) {
        if (b.kind !== "choice") continue;
        if (b.asked) out.push(b);
        for (const arm of b.arms) choicesIn(arm.blocks, out);
    }
    return out;
}

describe.skipIf(!present)("the EN corpus through the book walk", () => {
    it("drops nothing silently and prints every option of every decision", () => {
        const files = readdirSync(DIR).filter((f) => f.endsWith(".json"));
        const totals: Record<string, number> = {};
        const dropped: Record<string, number> = {};
        let decisions = 0;
        let options = 0;
        let words = 0;
        for (const f of files) {
            const script = JSON.parse(readFileSync(join(DIR, f), "utf8")) as StoryScript;
            const section = sectionOf(script, null, { nickname: "Doctor", images: "cg+bg", branches: "all" });
            for (const [kind, [inCount, outCount]] of Object.entries(ledger(section.stats))) {
                expect(outCount, `${script.id} ${kind}`).toBe(inCount);
                totals[kind] = (totals[kind] ?? 0) + inCount;
            }
            for (const [k, v] of Object.entries(section.stats.dropped)) dropped[k] = (dropped[k] ?? 0) + v;
            words += section.stats.words;
            // Every decision command with values is a choice block with all its options, in order.
            const asked = choicesIn(section.blocks);
            const commands = script.commands.filter((c) => c.kind === "decision" && (c.args.values ?? "").trim() !== "");
            expect(asked.length, script.id).toBe(commands.length);
            const text = toText({ meta: { title: script.id, identifier: "", server: "en", language: "en", scopeId: "", options: { nickname: "Doctor", images: "cg+bg", branches: "all" }, cover: {}, description: "" }, parts: [{ id: "g", title: "g", sections: [section] }] });
            commands.forEach((c, i) => {
                const texts = (c.args.options ?? "").split(";");
                const values = (c.args.values ?? "").split(";");
                expect(asked[i].options.length, `${script.id}:${c.line}`).toBe(Math.min(texts.length, values.length));
                for (const t of texts) {
                    const plain = plainStoryText(renderLine(t, "Doctor")).trim();
                    if (plain !== "") expect(text.includes(plain), `${script.id}:${c.line} "${plain}"`).toBe(true);
                }
                options += values.length;
            });
            decisions += commands.length;
        }
        // The census the register quotes; printed so a data refresh shows what moved.
        console.log(JSON.stringify({ scripts: files.length, decisions, options, words, totals, dropped }));
        expect(files.length).toBeGreaterThan(1000);
    }, 120_000);
});
