// Prepared answers from the bank, placed on the gatePos scale: the quick look and the popular list's seeds.
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { GATE_POS, MANIFEST, TREVOR, VERSION } from "./config.mjs";

/** A passage cut to at most 600 characters at a line break, like the worker's citations. */
function cut600(text) {
    if (text.length <= 600) return text;
    const at = text.lastIndexOf("\n", 600);
    return at > 300 ? text.slice(0, at) : text.slice(0, 600);
}
export const DECLINES = /^(the (provided|given) (text|passages?)|i cannot|i can't|there is no|it is not)/i;
const STOP = new Set("what when where which while with from that this they them their there does have been were will would could should about into after before trevor".split(" "));
const words = (s) => new Set((s.toLowerCase().match(/[a-z0-9'’]{4,}/g) ?? []).filter((w) => !STOP.has(w)));
/** Bank answers whose every passage is placed on the gatePos scale: { qid, question, text, gatePos, citations, words }. */
export const BANK = (() => {
    const file = path.join(TREVOR, "artifacts/bank/answers.jsonl");
    const chunksFile = path.join(TREVOR, "artifacts/p3a/chunks.jsonl");
    if (!fs.existsSync(file) || !fs.existsSync(chunksFile)) return [];
    const rows = [];
    for (const line of fs.readFileSync(file, "utf8").split("\n")) {
        if (!line.trim()) continue;
        try {
            const r = JSON.parse(line);
            // Declines and questions still holding a template ("Dr. {@nickname}") are never offered.
            if (!r.answer || !r.cited?.length || !r.passages?.length || DECLINES.test(r.answer) || /\{@/.test(r.question + r.answer)) continue;
            const pos = r.passages.map((c) => GATE_POS.get(c.split("#")[0]));
            if (pos.some((p) => p === undefined)) continue;
            rows.push({ ...r, gatePos: Math.max(...pos) });
        } catch {}
    }
    const need = new Set(rows.flatMap((r) => r.cited));
    const chunks = new Map();
    for (const line of fs.readFileSync(chunksFile, "utf8").split("\n")) {
        const id = line.match(/^\{"chunkId":"([^"]+)"/)?.[1];
        if (id && need.has(id)) chunks.set(id, JSON.parse(line));
    }
    return rows.map((r) => ({
        qid: r.qid,
        question: r.question,
        text: r.answer,
        gatePos: r.gatePos,
        words: words(r.question),
        citations: r.cited
            .filter((id) => chunks.has(id))
            .map((id) => {
                const c = chunks.get(id);
                return { n: r.passages.indexOf(id) + 1, storyId: c.storyId, chunkId: id, lineStart: c.lineStart, lineEnd: c.lineEnd, text: cut600(c.text) };
            })
            .sort((a, b) => a.n - b.n),
    }));
})();

/** The quick look (contract v1.1 section 1b): the closest bank answer inside the era, as a passages answer. */
export function quickLook(question, horizon) {
    const qw = words(question);
    // A name (a capitalized word after the first) counts three words.
    const names = new Set((question.match(/(?<=\S\s+)[A-Z][\p{L}'’-]{2,}/gu) ?? []).map((w) => w.toLowerCase()));
    let best = null;
    let score = -1;
    for (const e of BANK) {
        if (e.gatePos > horizon.eraPos) continue;
        let s = 0;
        for (const w of e.words) if (qw.has(w)) s += names.has(w) ? 3 : 1;
        if (s > score || (s === score && e.gatePos > best.gatePos)) [best, score] = [e, s];
    }
    // A prepared answer that shares one plain word with the question is no quick look at all.
    if (score < 2) best = null;
    const text = best
        ? `Trevor is busy, so this is the closest prepared answer, to "${best.question}":\n\n${best.text}\n\nThe full answer to your question is on its way.`
        : "Trevor is busy, and no prepared answer within your reading so far matches this question. The full answer is on its way.";
    return { id: crypto.randomUUID(), version: VERSION, server: "en", eraPos: horizon.eraPos, rung: "passages", source: "passages", answer: { answer: text, citations: best?.citations ?? [] } };
}

/** Bank answers inside an era as cached answers, newest gatePos first (the popular seeds). */
const seedIds = new Map();
export function seeds(eraPos, n) {
    return BANK.filter((e) => e.gatePos <= eraPos && e.citations.length > 0)
        .sort((a, b) => b.gatePos - a.gatePos)
        .slice(0, n)
        .map((e) => {
            if (!seedIds.has(e.qid)) seedIds.set(e.qid, crypto.randomUUID());
            const body = { answer: e.text, rung: "cpu", profile: "full", source: "live", citations: e.citations, route: null, form: null, corpusVersion: MANIFEST.corpusVersion };
            return { question: e.question, answer: { id: seedIds.get(e.qid), version: VERSION, server: "en", eraPos, rung: "cpu", source: "live", answer: body }, askers: 2 };
        });
}
