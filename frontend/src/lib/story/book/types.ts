/**
 * THE BOOK IR: a story, a chapter or a hand-picked set of stories as a tree
 * of printable blocks, built by a LINEAR walk of `StoryScript.commands` and
 * rendered by `html.ts`, `epub.ts` and `plain.ts`. Nothing here knows about
 * the DOM, so the same tree can be rendered in the browser, in a test and,
 * later, on a server route. Plan and measured numbers: `docs/story-export-plan.md`.
 */
import type { TextNode } from "../text";

/** Which images a book carries. Scene rules print either way; `cg+bg` adds the background as a picture. */
export type ImageMode = "cg" | "cg+bg" | "none";
/** `all` prints every option of every decision with its branch; `path` prints the one the reader took. */
export type BranchMode = "all" | "path";
export type Typeface = "inter" | "opendyslexic" | "device";
/** A CG at full book size (<= 1,200 px), or a scene background as a thumbnail (<= 600 px). */
export type ImageVariant = "full" | "thumb";

export interface BookOptions {
    /** The Doctor's name, already resolved (`resolveNickname`). */
    nickname: string;
    images: ImageMode;
    branches: BranchMode;
    /**
     * Saved choices per story id, the reader's `pos[id].choices` (decision
     * ordinal -> value). Read only when `branches` is `path`; a story with no
     * entry takes the FIRST option at every decision.
     */
    choices?: Readonly<Record<string, Readonly<Record<number, string>>>>;
}

/**
 * `groups` is an ordered run of whole groups (an arc, a storyline, a range of
 * a reading order): one part per group, titled `title`, identified by `id`.
 */
export type BookScope = { kind: "story"; storyId: string } | { kind: "group"; groupId: string } | { kind: "selection"; ids: readonly string[] } | { kind: "groups"; ids: readonly string[]; title: string; id: string };

/** One option of a decision, in the order the game lists them. `value` is the script's own, and is NOT always 1..n. */
export interface ChoiceOption {
    value: string;
    nodes: TextNode[];
}

/**
 * The lines a predicate gates. `values` are the option values it admits, in
 * this choice's own vocabulary; several values is a gate one or more options
 * share (`references=1;2` before option 3's own lines).
 */
export interface ChoiceArm {
    values: string[];
    blocks: Block[];
}

export type Block =
    | { kind: "line"; id: string; speaker: string; nodes: TextNode[] }
    | { kind: "narration"; id: string; nodes: TextNode[] }
    /** A subtitle or a sticker: centred, newlines kept as written. */
    | { kind: "overlay"; id: string; style: "subtitle" | "sticker"; nodes: TextNode[] }
    /** Every background CHANGE. `url` is the resolved asset path, absent when the script names a file the wire has none for. */
    | { kind: "scene"; id: string; name: string; url?: string }
    /** A CG, a CG piece, an item close-up or a panel strip: one or several pictures shown together. */
    | { kind: "figure"; id: string; source: "image" | "cgitem" | "showitem" | "largebg" | "gridbg"; images: { name: string; url: string }[] }
    | ChoiceBlock
    | { kind: "cutscene"; id: string; res: string };

/**
 * A decision and what it gates. `asked` is false for a gate that reopens an
 * EARLIER decision after its join (the game lists no options there, it only
 * shows or hides the lines). With `branches: "path"`, `chosen` is the value
 * the walk followed and `arms` hold only the lines on that path.
 */
export interface ChoiceBlock {
    kind: "choice";
    id: string;
    asked: boolean;
    /** On a recall (`asked: false`), the id of the decision it reopens. */
    recalls?: string;
    options: ChoiceOption[];
    arms: ChoiceArm[];
    chosen?: string;
}

export type BlockKind = Block["kind"];

/**
 * Accounting for one story, so that nothing is dropped silently: every
 * command kind that can print is counted IN, every block OUT, and every
 * command that was consumed without a block is counted under its reason.
 */
export interface SectionStats {
    words: number;
    in: Record<string, number>;
    out: Partial<Record<BlockKind, number>>;
    dropped: Record<string, number>;
}

export interface Section {
    /** The story id; also the anchor and the file name inside an EPUB. */
    id: string;
    code?: string;
    /** `Before Operation`, `After Operation`, `Interlude`. */
    tag?: string;
    name: string;
    synopsis?: string;
    blocks: Block[];
    stats: SectionStats;
}

/** A group's stories inside a book. A book of one group has one part. */
export interface Part {
    id: string;
    title: string;
    sections: Section[];
}

export interface BookMeta {
    title: string;
    /** `urn:myrtle:story:<server>:<scope-id>`, stable across exports of the same scope. */
    identifier: string;
    server: string;
    language: string;
    scopeId: string;
    options: BookOptions;
    /** Art for the cover, as asset paths: the logotype over the key visual, or a cover alone. */
    cover: { titleImageUrl?: string; bannerUrl?: string; coverUrl?: string };
    /** Each part's key visual (the group's banner, else its cover), by group id, for part-title pages. */
    partArt?: Record<string, string | undefined>;
    /** The synopses, joined, for `dc:description`. */
    description: string;
}

export interface Book {
    meta: BookMeta;
    parts: Part[];
}
