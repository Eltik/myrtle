/**
 * ONE STORY AS A LIST OF BLOCKS, by a LINEAR walk of its commands.
 *
 * The engine is a player: it follows one path and skips every branch it did
 * not take. A book prints all of them, so this walks the commands in file
 * order and builds a TREE out of `decision` and the `predicate` gates after
 * it. What the walk shows and hides is still the engine's own rule wherever
 * the engine has one (a repeated sticker id hides, a subtitle off the canvas
 * is rejected, a `largebg` with an unresolved panel draws nothing), so the
 * book never prints a line the game never shows.
 *
 * THE GATE RULE, from the 3,883 EN decisions (`docs/story-export-plan.md`,
 * and the census in the 2026-09-30 register entry). 3,516 are followed at
 * once by a gate naming every option (the options are only the Doctor's
 * reply); 280 gate each option in turn and join; the rest nest. Values are
 * the SCRIPT's, not 1..n: act12d0_01 numbers its decisions 1;2, 3;4, 5;6 so a
 * gate on `2` after the 3;4 decision reopens the OUTER decision's option 2,
 * and `3;4;5;6` joins both inner decisions and the outer one with them. So a
 * gate is resolved against a STACK of open decisions:
 *
 *   1. JOIN: while the innermost open decision is COVERED by the gate (every
 *      option named, or its branch ends in a nested decision that is), close it.
 *   2. ARM: the innermost open decision the gate names opens a new arm with
 *      the named values; anything deeper is closed.
 *   3. After a join with nothing named, the lines continue where they are.
 *   4. RECALL: a gate naming only the LATEST decision after it closed reopens
 *      it as a block of its own (`asked: false`), in place, because that is
 *      where the game shows those lines. An older decision cannot be recalled:
 *      the engine gates on the last choice made.
 *   5. Otherwise the gate names no decision at all (main_15-06 gates `2`
 *      after a decision whose three options are all `1`): the game never
 *      shows those lines, and they are counted as `gate:unreachable`.
 *
 * An EMPTY gate (`[predicate]` with no references, 3 in main_01-03 alone)
 * joins everything. The ENGINE reads it differently: it splits "" into
 * [""], no choice matches, and it SKIPS the lines after it, so the reader
 * hides "Sounds good." and two more lines from every reader of main_01-03.
 * The book prints them; the engine is not changed here (register, 2026-09-30).
 */
import type { StoryEntry } from "#/types/generated/StoryEntry";
import type { StoryScript } from "#/types/generated/StoryScript";
import { key } from "../args";
import { CANVAS_H, CANVAS_W } from "../canvas";
import { overlayFrom } from "../scene";
import { plainStoryText, renderLine, substitute, type TextNode } from "../text";
import type { Block, BlockKind, BookOptions, ChoiceArm, ChoiceBlock, Section, SectionStats } from "./types";

/**
 * One open decision. `background` and `image` are the scene AT the decision:
 * every arm starts from it, so a background change in option 1's branch is
 * still a change in option 2's. `changed` records that some arm moved the
 * scene, which makes the scene after the join unknown.
 */
interface Frame {
    choice: ChoiceBlock;
    arm: ChoiceArm | null;
    background: string | undefined;
    image: string | undefined;
    changed: boolean;
}

/** The command kinds that can put something on the page, as the accounting counts them. */
export const PRINTING_KINDS = ["name", "multiline", "text", "narration", "dialog", "voicewithin", "subtitle", "sticker", "decision", "background", "image", "cgitem", "showitem", "largebg", "gridbg", "video"] as const;

function words(nodes: TextNode[]): number {
    const t = plainStoryText(nodes).trim();
    return t === "" ? 0 : t.split(/\s+/).length;
}

function optionValues(choice: ChoiceBlock): string[] {
    return [...new Set(choice.options.map((o) => o.value))];
}

/** Every option of `choice` is named by `refs`, or its branch ends in a nested decision that is. */
function covered(choice: ChoiceBlock, refs: ReadonlySet<string>): boolean {
    return optionValues(choice).every((v) => refs.has(v) || choice.arms.some((arm) => arm.values.includes(v) && arm.blocks.some((b) => b.kind === "choice" && b.asked && covered(b, refs))));
}

function names(choice: ChoiceBlock, refs: ReadonlySet<string>): string[] {
    return optionValues(choice).filter((v) => refs.has(v));
}

/**
 * Build one section. `entry` supplies the code, phase and title the library
 * shows; without it the script's own name is the title.
 */
export function sectionOf(script: StoryScript, entry: Pick<StoryEntry, "code" | "avgTag" | "name"> | null, options: BookOptions): Section {
    const nickname = options.nickname;
    const assets = script.assets;
    const stats: SectionStats = { words: 0, in: {}, out: {}, dropped: {} };
    const count = (bag: Record<string, number>, k: string) => {
        bag[k] = (bag[k] ?? 0) + 1;
    };

    const root: Block[] = [];
    const stack: Frame[] = [];
    const closed: ChoiceBlock[] = [];
    let lastAsked: ChoiceBlock | undefined;
    let limbo = false;
    let seq = 0;
    const nextId = () => `b${++seq}`;
    let background: string | undefined;
    let image: string | undefined;
    const stickers = new Set<string>();

    const close = (frame: Frame | undefined) => {
        if (!frame) return;
        if (frame.choice.asked) closed.push(frame.choice);
        if (frame.changed) {
            // Which scene the reader is in now depends on the path: the next background prints.
            background = undefined;
            image = undefined;
        } else {
            background = frame.background;
            image = frame.image;
        }
    };
    const open = (frame: Frame, values: string[]) => {
        // A gate that names the same options as the arm before it CONTINUES that
        // arm (main_15-06: `1;1;1` joins an inner decision and carries on in the
        // outer option 1), so one option never gets two headings in a row.
        const last = frame.choice.arms[frame.choice.arms.length - 1];
        const same = last !== undefined && last.values.length === values.length && last.values.every((v) => values.includes(v));
        const arm = same ? last : { values, blocks: [] };
        if (!same) frame.choice.arms.push(arm);
        frame.arm = arm;
        background = frame.background;
        image = frame.image;
    };
    const moved = () => {
        for (const f of stack) f.changed = true;
    };
    /** Where the next block goes: the open arm, or the flow. A decision with no arm open yet closes, because what follows it is shown to everyone. */
    const container = (): Block[] | null => {
        if (limbo) return null;
        while (stack.length > 0 && stack[stack.length - 1].arm === null) close(stack.pop());
        const top = stack[stack.length - 1];
        return top?.arm ? top.arm.blocks : root;
    };
    const emit = (block: Block): void => {
        const into = container();
        if (into) into.push(block);
        else count(stats.dropped, `gate:unreachable:${block.kind}`);
    };
    const drop = (reason: string) => count(stats.dropped, reason);

    const predicate = (raw: string | undefined) => {
        limbo = false;
        const refs = new Set(
            (raw ?? "")
                .split(";")
                .map((s) => s.trim())
                .filter((s) => s !== ""),
        );
        if (refs.size === 0) {
            while (stack.length > 0) close(stack.pop());
            return;
        }
        let joined = false;
        while (stack.length > 0 && covered(stack[stack.length - 1].choice, refs)) {
            close(stack.pop());
            joined = true;
        }
        for (let i = stack.length - 1; i >= 0; i -= 1) {
            const frame = stack[i];
            const named = names(frame.choice, refs);
            if (named.length === 0) continue;
            while (stack.length > i + 1) close(stack.pop());
            open(frame, named);
            return;
        }
        if (joined) return;
        // Only the LATEST decision can be recalled: the engine gates on the last
        // choice made, so a gate naming an older decision's value never shows.
        const source = closed[closed.length - 1];
        if (source && source === lastAsked) {
            const named = names(source, refs);
            if (named.length === 0) {
                limbo = true;
                return;
            }
            const arm: ChoiceArm = { values: named, blocks: [] };
            const recall: ChoiceBlock = { kind: "choice", id: nextId(), asked: false, recalls: source.id, options: source.options, arms: [arm] };
            const into = container();
            if (!into) {
                limbo = true;
                return;
            }
            into.push(recall);
            stack.push({ choice: recall, arm, background, image, changed: false });
            return;
        }
        limbo = true;
    };

    for (const cmd of script.commands) {
        const a = cmd.args;
        const kind = cmd.kind;
        if ((PRINTING_KINDS as readonly string[]).includes(kind)) count(stats.in, kind);
        switch (kind) {
            case "predicate":
                predicate(a.references);
                break;
            case "decision": {
                const values = (a.values ?? "").split(";").map((s) => s.trim());
                if (values.length === 0 || (values.length === 1 && values[0] === "")) {
                    drop("decision:novalues");
                    break;
                }
                // The engine shows a decision whatever gate it sits under.
                limbo = false;
                // One EN option carries a literal ";" (main_08-14_end line 378: three
                // values, four pieces), so the pieces past the last value belong to it.
                // The other way round, 8 EN decisions list MORE values than options
                // (main_06-02 line 198: one option, values 1;2;3); the game shows the
                // options, so the extra values have no option and print nothing.
                const pieces = (a.options ?? "").split(";");
                const texts = pieces.length > values.length ? [...pieces.slice(0, values.length - 1), pieces.slice(values.length - 1).join(";")] : pieces;
                const choice: ChoiceBlock = { kind: "choice", id: nextId(), asked: true, options: texts.slice(0, values.length).map((text, i) => ({ value: values[i], nodes: renderLine(text, nickname) })), arms: [] };
                emit(choice);
                lastAsked = choice;
                stack.push({ choice, arm: null, background, image, changed: false });
                break;
            }
            case "name":
            case "multiline": {
                const text = cmd.text ?? "";
                if (text.trim() === "") {
                    drop("text:empty");
                    break;
                }
                const speaker = substitute((a.name ?? "").trim(), nickname).trim();
                const nodes = renderLine(text, nickname);
                // `[name=""]` has no plate: the reader draws it as narration, and so does the book.
                emit(speaker === "" ? { kind: "narration", id: nextId(), nodes } : { kind: "line", id: nextId(), speaker, nodes });
                break;
            }
            case "text":
            case "narration":
            case "dialog":
            case "voicewithin": {
                const text = cmd.text ?? "";
                if (text.trim() === "") {
                    // A bare `[dialog]` hides the box: not a line.
                    if (kind === "text" || kind === "narration") drop("text:empty");
                    else count(stats.in, `${kind}:bare`);
                    break;
                }
                emit({ kind: "narration", id: nextId(), nodes: renderLine(text, nickname) });
                break;
            }
            case "subtitle": {
                const text = a.text;
                if (text === undefined) {
                    count(stats.in, "subtitle:bare");
                    break;
                }
                const o = overlayFrom(a, text, nickname);
                if (o.x < 0 || o.x > CANVAS_W || o.y < 0 || o.y > CANVAS_H) {
                    drop("subtitle:offcanvas");
                    break;
                }
                emit({ kind: "overlay", id: nextId(), style: "subtitle", nodes: renderLine(text, nickname) });
                break;
            }
            case "sticker": {
                const id = a.id ?? "st";
                const text = a.text;
                if (text === undefined) {
                    stickers.delete(id);
                    count(stats.in, "sticker:bare");
                    break;
                }
                const multi = (a.multi ?? "").trim().toLowerCase();
                if (stickers.has(id) && multi !== "true" && multi !== "1") {
                    // The engine HIDES an existing id repeated without `multi`, text and all.
                    stickers.delete(id);
                    drop("sticker:repeatHides");
                    break;
                }
                stickers.add(id);
                emit({ kind: "overlay", id: nextId(), style: "sticker", nodes: renderLine(text, nickname) });
                break;
            }
            case "stickerclear":
                stickers.clear();
                break;
            case "background": {
                const name = a.image === undefined ? undefined : key(a.image);
                if (name === undefined || name === "") {
                    background = undefined;
                    moved();
                    count(stats.in, "background:bare");
                    break;
                }
                if (name === background) {
                    drop("background:same");
                    break;
                }
                background = name;
                moved();
                emit({ kind: "scene", id: nextId(), name, url: assets.backgrounds[name] });
                break;
            }
            case "image": {
                const name = a.image === undefined ? undefined : key(a.image);
                if (name === undefined || name === "") {
                    image = undefined;
                    moved();
                    count(stats.in, "image:bare");
                    break;
                }
                if (name === image) {
                    drop("image:same");
                    break;
                }
                image = name;
                moved();
                const url = assets.images[name];
                if (!url) {
                    drop("image:unresolved");
                    break;
                }
                emit({ kind: "figure", id: nextId(), source: "image", images: [{ name, url }] });
                break;
            }
            case "cgitem":
            case "showitem": {
                const name = key(a.image ?? "");
                if (name === "") {
                    count(stats.in, `${kind}:bare`);
                    break;
                }
                const url = assets.images[name] ?? assets.backgrounds[name];
                if (!url) {
                    drop(`${kind}:unresolved`);
                    break;
                }
                emit({ kind: "figure", id: nextId(), source: kind, images: [{ name, url }] });
                break;
            }
            case "largebg":
            case "gridbg": {
                const group = a.imagegroup ?? a.cggroup;
                if (group === undefined || group.trim() === "") {
                    count(stats.in, `${kind}:bare`);
                    break;
                }
                const list = group
                    .split("/")
                    .map((n) => key(n))
                    .filter((n) => n !== "");
                const images = list.map((name) => ({ name, url: assets.backgrounds[name] ?? assets.images[name] ?? "" }));
                // The engine draws nothing unless every panel resolves.
                if (images.length === 0 || images.some((i) => i.url === "")) {
                    drop(`${kind}:unresolved`);
                    break;
                }
                emit({ kind: "figure", id: nextId(), source: kind, images });
                break;
            }
            case "video": {
                const res = (a.res ?? "").trim();
                const sources = res === "" ? undefined : assets.videos?.[res];
                if (!sources || (sources.webmUrl === undefined && sources.mp4Url === undefined)) {
                    // 13 of the 25 EN clip names have no file; the reader skips them too.
                    drop("video:unresolved");
                    break;
                }
                emit({ kind: "cutscene", id: nextId(), res });
                break;
            }
            default:
                break;
        }
    }

    const blocks = options.branches === "path" ? prunePath(root, options.choices?.[script.id] ?? {}) : root;
    tally(blocks, stats);
    const title = entry?.name?.trim() || script.name;
    return { id: script.id, code: entry?.code, tag: entry?.avgTag, name: title, synopsis: script.synopsis?.trim() || undefined, blocks, stats };
}

/** Count blocks and words over the final tree. */
function tally(blocks: readonly Block[], stats: SectionStats): void {
    const bump = (k: BlockKind) => {
        stats.out[k] = (stats.out[k] ?? 0) + 1;
    };
    for (const b of blocks) {
        if (b.kind === "choice") {
            if (b.asked) bump("choice");
            for (const o of b.chosen !== undefined ? b.options.filter((o) => o.value === b.chosen).slice(0, 1) : b.asked ? b.options : []) stats.words += words(o.nodes);
            for (const arm of b.arms) tally(arm.blocks, stats);
            continue;
        }
        bump(b.kind);
        if (b.kind === "line" || b.kind === "narration" || b.kind === "overlay") stats.words += words(b.nodes);
    }
}

/**
 * The reader's own path through a tree. Decisions are numbered the way the
 * ENGINE numbers them, in the order the path reaches them, so a saved
 * `choices` map lines up with the tree; a missing or stale value takes the
 * first option, which is what the engine does with no choice.
 */
export function prunePath(blocks: readonly Block[], saved: Readonly<Record<number, string>>): Block[] {
    const ctx = { ordinal: 0, chosen: new Map<string, string>() };
    const walk = (list: readonly Block[]): Block[] => {
        const out: Block[] = [];
        for (const b of list) {
            if (b.kind !== "choice") {
                out.push(b);
                continue;
            }
            let value: string | undefined;
            if (b.asked) {
                const ordinal = ctx.ordinal;
                ctx.ordinal += 1;
                const wanted = saved[ordinal];
                value = wanted !== undefined && b.options.some((o) => o.value === wanted) ? wanted : b.options[0]?.value;
                if (value !== undefined) ctx.chosen.set(b.id, value);
            } else {
                value = b.recalls === undefined ? undefined : ctx.chosen.get(b.recalls);
            }
            if (value === undefined) continue;
            const v = value;
            const arms = b.arms.filter((arm) => arm.values.includes(v)).map((arm) => ({ values: arm.values, blocks: walk(arm.blocks) }));
            if (!b.asked && arms.length === 0) continue;
            out.push({ ...b, chosen: v, arms });
        }
        return out;
    };
    return walk(blocks);
}
