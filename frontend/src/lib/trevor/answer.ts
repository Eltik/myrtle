/**
 * ONE ANSWER, as the panel draws it.
 *
 * `TrevorAnswer.answer` is the worker's structured result (contract v1,
 * section 3) carried as opaque JSON: `{answer, rung, profile, source,
 * citations: [{n, storyId, chunkId, lineStart, lineEnd, text}], ...}`. This
 * module reads the parts the reader needs and turns the answer text into
 * blocks a component can render as React nodes, never as HTML: the model
 * writes light Markdown (`**bold**`, `*   ` bullets) and citation markers
 * (`[4]`, `[1, 4]`).
 */
import type { TrevorAnswer } from "#/types/generated/TrevorAnswer";
import type { TrevorAnswerSource } from "#/types/generated/TrevorAnswerSource";
import type { TrevorRung } from "#/types/generated/TrevorRung";

export interface ITrevorCitation {
    n: number;
    storyId: string;
    chunkId: string;
    /** 1-based script line, the same numbering as the reader's `?line=`. */
    lineStart: number;
    lineEnd: number;
    text: string;
}

export interface IParsedAnswer {
    /** The answer id feedback is sent against. */
    id: string;
    text: string;
    source: TrevorAnswerSource;
    rung: TrevorRung;
    eraPos: number;
    citations: ITrevorCitation[];
}

function asRecord(v: unknown): Record<string, unknown> | null {
    return v && typeof v === "object" && !Array.isArray(v) ? (v as Record<string, unknown>) : null;
}

function asCitation(v: unknown): ITrevorCitation | null {
    const c = asRecord(v);
    if (!c) return null;
    const n = Number(c.n);
    if (!Number.isFinite(n) || typeof c.storyId !== "string") return null;
    return {
        n,
        storyId: c.storyId,
        chunkId: typeof c.chunkId === "string" ? c.chunkId : "",
        lineStart: Number.isFinite(Number(c.lineStart)) ? Number(c.lineStart) : 0,
        lineEnd: Number.isFinite(Number(c.lineEnd)) ? Number(c.lineEnd) : 0,
        text: typeof c.text === "string" ? c.text : "",
    };
}

/** The reader's view of a stored answer. A body with no text reads as an empty answer, never a throw. */
export function parseAnswer(a: TrevorAnswer): IParsedAnswer {
    const body = asRecord(a.answer);
    const citations = Array.isArray(body?.citations) ? body.citations.map(asCitation).filter((c): c is ITrevorCitation => c !== null) : [];
    citations.sort((x, y) => x.n - y.n);
    return {
        id: a.id,
        text: typeof body?.answer === "string" ? body.answer : "",
        source: a.source,
        rung: a.rung,
        eraPos: a.eraPos,
        citations,
    };
}

export type AnswerInline = { kind: "text"; text: string } | { kind: "bold"; text: string } | { kind: "cite"; ns: number[] };
type AnswerBlock = { kind: "p"; inlines: AnswerInline[] } | { kind: "ul"; items: AnswerInline[][] };

const CITE = /\[(\d+(?:\s*,\s*\d+)*)\]/g;
const BULLET = /^\s*(?:[*\-•]|\d+[.)])\s+/;

function plainAndBold(text: string, out: AnswerInline[]): void {
    let last = 0;
    for (const m of text.matchAll(/\*\*(.+?)\*\*/g)) {
        const at = m.index ?? 0;
        if (at > last) out.push({ kind: "text", text: text.slice(last, at) });
        out.push({ kind: "bold", text: m[1] ?? "" });
        last = at + m[0].length;
    }
    if (last < text.length) out.push({ kind: "text", text: text.slice(last) });
}

/** One line's inline runs: text, `**bold**`, and `[n]` / `[n, m]` markers. */
export function answerInlines(line: string): AnswerInline[] {
    const out: AnswerInline[] = [];
    let last = 0;
    for (const m of line.matchAll(CITE)) {
        const at = m.index ?? 0;
        if (at > last) plainAndBold(line.slice(last, at), out);
        out.push({ kind: "cite", ns: (m[1] ?? "").split(",").map((s) => Number(s.trim())) });
        last = at + m[0].length;
    }
    if (last < line.length) plainAndBold(line.slice(last), out);
    return out;
}

/** Paragraphs and bullet lists, split on blank lines and bullet markers. */
export function answerBlocks(text: string): AnswerBlock[] {
    const blocks: AnswerBlock[] = [];
    let para: string[] = [];
    let list: AnswerInline[][] | null = null;
    const flushPara = () => {
        if (para.length > 0) blocks.push({ kind: "p", inlines: answerInlines(para.join(" ")) });
        para = [];
    };
    const flushList = () => {
        if (list && list.length > 0) blocks.push({ kind: "ul", items: list });
        list = null;
    };
    for (const raw of text.split(/\r?\n/)) {
        const line = raw.trim();
        if (line === "") {
            flushPara();
            flushList();
            continue;
        }
        if (BULLET.test(raw)) {
            flushPara();
            list ??= [];
            list.push(answerInlines(line.replace(BULLET, "")));
            continue;
        }
        flushList();
        para.push(line);
    }
    flushPara();
    flushList();
    return blocks;
}
