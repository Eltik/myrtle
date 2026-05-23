import { parseInline } from "./inline";
import type { BlockNode, InlineNode, Root } from "./types";

const HEADING_RE = /^(#{1,3})\s+(.*)$/;
const HR_RE = /^[ \t]*(?:-{3,}|_{3,}|\*{3,})[ \t]*$/;
const FENCE_RE = /^[ \t]*```([a-zA-Z0-9_+.-]*)[ \t]*$/;
const ORDERED_RE = /^([ \t]*)(\d{1,9})[.)][ \t]+(.*)$/;
const UNORDERED_RE = /^([ \t]*)[-*+][ \t]+(.*)$/;
const BLOCKQUOTE_RE = /^[ \t]*>[ \t]?(.*)$/;

export function tokenize(src: string): Root {
    const lines = src.replace(/\r\n?/g, "\n").split("\n");
    const blocks: BlockNode[] = [];
    let i = 0;

    while (i < lines.length) {
        const line = lines[i] ?? "";

        if (!line.trim()) {
            i++;
            continue;
        }

        // HR
        if (HR_RE.test(line) && !UNORDERED_RE.test(line)) {
            blocks.push({ type: "hr" });
            i++;
            continue;
        }

        // ATX heading
        const headingMatch = HEADING_RE.exec(line);
        if (headingMatch) {
            const hashes = headingMatch[1] ?? "";
            const depth = (hashes.length || 1) as 1 | 2 | 3;
            const text = (headingMatch[2] ?? "").replace(/\s+#+\s*$/, "").trim();
            blocks.push({ type: "heading", depth, children: parseInline(text) });
            i++;
            continue;
        }

        // Fenced code
        const fenceMatch = FENCE_RE.exec(line);
        if (fenceMatch) {
            const lang = fenceMatch[1] || null;
            const start = i + 1;
            let end = start;
            while (end < lines.length && !FENCE_RE.test(lines[end] ?? "")) end++;
            const value = lines.slice(start, end).join("\n");
            blocks.push({ type: "code", lang, value });
            i = end < lines.length ? end + 1 : end;
            continue;
        }

        // Blockquote
        if (BLOCKQUOTE_RE.test(line)) {
            const collected: string[] = [];
            while (i < lines.length) {
                const m = BLOCKQUOTE_RE.exec(lines[i] ?? "");
                if (!m) {
                    // Lazy continuation: a non-blank line without `>` continues the blockquote
                    const cur = lines[i] ?? "";
                    if (cur.trim() && !isBlockStarter(cur)) {
                        collected.push(cur);
                        i++;
                        continue;
                    }
                    break;
                }
                collected.push(m[1] ?? "");
                i++;
            }
            blocks.push({ type: "blockquote", children: tokenize(collected.join("\n")) });
            continue;
        }

        // List
        const listResult = tryParseList(lines, i);
        if (listResult) {
            blocks.push(listResult.list);
            i = listResult.end;
            continue;
        }

        // Paragraph
        const paraStart = i;
        const paraLines: string[] = [line];
        i++;
        while (i < lines.length) {
            const cur = lines[i] ?? "";
            if (!cur.trim()) break;
            if (isBlockStarter(cur)) break;
            paraLines.push(cur);
            i++;
        }
        blocks.push(buildParagraph(paraLines));
        // paraStart unused beyond tracking; quiet linter
        void paraStart;
    }

    return blocks;
}

function isBlockStarter(line: string): boolean {
    if (HR_RE.test(line) && !UNORDERED_RE.test(line)) return true;
    if (HEADING_RE.test(line)) return true;
    if (FENCE_RE.test(line)) return true;
    if (BLOCKQUOTE_RE.test(line)) return true;
    if (UNORDERED_RE.test(line)) return true;
    if (ORDERED_RE.test(line)) return true;
    return false;
}

function buildParagraph(lines: string[]): BlockNode {
    const children: InlineNode[] = [];
    for (let idx = 0; idx < lines.length; idx++) {
        if (idx > 0) children.push({ type: "break" });
        const segment = parseInline(lines[idx] ?? "");
        for (const n of segment) children.push(n);
    }
    return { type: "paragraph", children };
}

function tryParseList(lines: string[], start: number): { list: BlockNode; end: number } | null {
    const first = lines[start] ?? "";
    const orderedFirst = ORDERED_RE.exec(first);
    const unorderedFirst = UNORDERED_RE.exec(first);
    if (!orderedFirst && !unorderedFirst) return null;

    const ordered = !!orderedFirst;
    const startNum = orderedFirst ? Number.parseInt(orderedFirst[2] ?? "1", 10) : 1;

    const items: BlockNode[][] = [];
    let i = start;

    while (i < lines.length) {
        const cur = lines[i] ?? "";
        const match = ordered ? ORDERED_RE.exec(cur) : UNORDERED_RE.exec(cur);
        if (!match) break;

        const initial = ordered ? (match[3] ?? "") : (match[2] ?? "");
        const itemLines: string[] = [initial];
        i++;

        // Gather continuation lines: indented (2+ spaces/tab) and not new list items
        while (i < lines.length) {
            const next = lines[i] ?? "";
            if (!next.trim()) {
                // blank line: peek ahead to see if a new same-type item follows; if not, stop
                const after = lines[i + 1] ?? "";
                const isNextItem = ordered ? ORDERED_RE.test(after) : UNORDERED_RE.test(after);
                if (isNextItem) {
                    i++;
                    break;
                }
                break;
            }
            if (ORDERED_RE.test(next) || UNORDERED_RE.test(next)) break;
            if (isBlockStarter(next) && !/^[ \t]+/.test(next)) break;
            itemLines.push(next.replace(/^[ \t]{1,4}/, ""));
            i++;
        }

        const itemBlocks = tokenize(itemLines.join("\n"));
        items.push(itemBlocks);
    }

    return {
        list: { type: "list", ordered, start: startNum, items },
        end: i,
    };
}
