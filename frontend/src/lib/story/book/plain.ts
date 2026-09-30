/**
 * THE BOOK AS MARKDOWN AND AS PLAIN TEXT (plan 3.2). `Speaker: line`; a
 * decision lists its options as `> option` lines and prints each branch
 * under its own heading, indented (a blockquote level in Markdown, two spaces
 * a level in text, so Markdown never turns a branch into a code block); a
 * background change is a bare `---` rule (the location has no name in the data but its file key); a CG is `![CG](url)`
 * in Markdown and `[CG] url` in text.
 */

import type { TextNode } from "../text";
import { storyLabel } from "./book";
import { type BookLabels, DEFAULT_LABELS } from "./render";
import type { Block, Book, ChoiceBlock, Section } from "./types";

export interface PlainOptions {
    labels?: BookLabels;
    /** An absolute URL for an asset path. */
    imageUrl?: (path: string) => string;
}

type Mode = "md" | "text";

/** Markdown's inline metacharacters, escaped so a line of dialogue never becomes emphasis or a link. */
function escapeMd(value: string): string {
    return value.replace(/([\\`*_[\]<>#|~])/g, "\\$1");
}

function inlineOf(nodes: readonly TextNode[], mode: Mode): string {
    return nodes
        .map((n) => {
            if (n.kind === "text") return mode === "md" ? escapeMd(n.value) : n.value;
            const inner = inlineOf(n.children, mode);
            if (n.kind === "paragraph") return `${inner}\n`;
            if (mode === "md" && n.kind === "inline" && inner.trim() !== "") {
                if (n.tag === "i") return `*${inner}*`;
                if (n.tag === "b") return `**${inner}**`;
            }
            return inner;
        })
        .join("")
        .replace(/\n+$/, "");
}

/** A paragraph's lines under the current indent; Markdown line breaks inside it are a trailing backslash. */
function para(text: string, mode: Mode, indent: string): string {
    const lines = text.split("\n");
    const joined = mode === "md" ? lines.join("\\\n") : lines.join("\n");
    return joined
        .split("\n")
        .map((l) => `${indent}${l}`.trimEnd())
        .join("\n");
}

function blocksOf(list: readonly Block[], mode: Mode, depth: number, labels: BookLabels, nickname: string, imageUrl: (p: string) => string, images: Book["meta"]["options"]["images"]): string[] {
    const indent = mode === "md" ? "> ".repeat(depth) : "  ".repeat(depth);
    const out: string[] = [];
    const who = (name: string) => (mode === "md" ? `**${escapeMd(name)}:**` : `${name}:`);
    for (const b of list) {
        switch (b.kind) {
            case "line":
                out.push(para(`${who(b.speaker)} ${inlineOf(b.nodes, mode)}`, mode, indent));
                break;
            case "narration":
                out.push(para(inlineOf(b.nodes, mode), mode, indent));
                break;
            case "overlay":
                out.push(para(mode === "md" ? `*${inlineOf(b.nodes, mode)}*` : inlineOf(b.nodes, mode), mode, indent));
                break;
            case "scene":
                // A rule only: the location's one name in the data is its file key.
                out.push(`${indent}---`.trimEnd());
                break;
            case "figure":
                if (images === "none") break;
                for (const i of b.images) out.push(mode === "md" ? `${indent}![CG](${imageUrl(i.url)})` : `${indent}[CG] ${imageUrl(i.url)}`);
                break;
            case "cutscene":
                out.push(mode === "md" ? `${indent}*[${escapeMd(labels.cutscene)}]*` : `${indent}[${labels.cutscene}]`);
                break;
            case "choice":
                out.push(...choiceOf(b, mode, depth, labels, nickname, imageUrl, images));
                break;
        }
    }
    return out;
}

function choiceOf(b: ChoiceBlock, mode: Mode, depth: number, labels: BookLabels, nickname: string, imageUrl: (p: string) => string, images: Book["meta"]["options"]["images"]): string[] {
    const indent = mode === "md" ? "> ".repeat(depth) : "  ".repeat(depth);
    const plainOption = (values: string[]) => b.options.filter((o) => values.includes(o.value)).map((o) => inlineOf(o.nodes, "text"));
    if (b.chosen !== undefined) {
        const picked = b.options.find((o) => o.value === b.chosen);
        const out: string[] = [];
        if (b.asked && picked) out.push(para(`${mode === "md" ? `**${escapeMd(nickname)}:**` : `${nickname}:`} ${inlineOf(picked.nodes, mode)}`, mode, indent));
        for (const arm of b.arms) out.push(...blocksOf(arm.blocks, mode, depth, labels, nickname, imageUrl, images));
        return out;
    }
    const out: string[] = [];
    if (b.asked) {
        // Markdown joins consecutive `>` lines into one paragraph, so its options are a quoted list.
        const optionLines = b.options.map((o) => `${indent}> ${mode === "md" ? "- " : ""}${inlineOf(o.nodes, mode)}`);
        out.push(mode === "md" ? `${indent}**${escapeMd(labels.choice)}:**` : `${indent}${labels.choice}:`, optionLines.join("\n"));
    }
    for (const arm of b.arms) {
        const heading = b.asked ? labels.ifChose(plainOption(arm.values)) : labels.earlier(plainOption(arm.values));
        out.push(mode === "md" ? `${indent}*${escapeMd(heading)}:*` : `${indent}${heading}:`);
        const inner = blocksOf(arm.blocks, mode, depth + 1, labels, nickname, imageUrl, images);
        if (inner.length > 0) out.push(...inner);
    }
    return out;
}

function sectionOf(s: Section, book: Book, mode: Mode, level: number, labels: BookLabels, imageUrl: (p: string) => string): string {
    const head: string[] = [];
    const title = storyLabel(s);
    if (mode === "md") {
        head.push(`${"#".repeat(level)} ${escapeMd(title)}`);
        if (s.tag) head.push(`*${escapeMd(s.tag)}*`);
        if (s.synopsis) head.push(`> ${escapeMd(s.synopsis).replace(/\n/g, "\n> ")}`);
    } else {
        head.push(`${title.toUpperCase()}\n${"=".repeat(Math.min(72, title.length))}`);
        if (s.tag) head.push(s.tag);
        if (s.synopsis) head.push(`${labels.synopsis}: ${s.synopsis}`);
    }
    const body = blocksOf(s.blocks, mode, 0, labels, book.meta.options.nickname, imageUrl, book.meta.options.images);
    return [...head, ...body].join("\n\n");
}

function render(book: Book, mode: Mode, opts: PlainOptions): string {
    const labels = opts.labels ?? DEFAULT_LABELS;
    const imageUrl = opts.imageUrl ?? ((p: string) => p);
    const multi = book.parts.length > 1;
    const chunks: string[] = [mode === "md" ? `# ${escapeMd(book.meta.title)}` : `${book.meta.title}\n${"#".repeat(Math.min(72, book.meta.title.length))}`];
    for (const part of book.parts) {
        if (multi) chunks.push(mode === "md" ? `## ${escapeMd(part.title)}` : `${part.title}\n${"*".repeat(Math.min(72, part.title.length))}`);
        for (const s of part.sections) chunks.push(sectionOf(s, book, mode, multi ? 3 : 2, labels, imageUrl));
    }
    chunks.push(mode === "md" ? `---\n\n*${escapeMd(labels.credit)} ${escapeMd(labels.madeWith)}*` : `--\n${labels.credit} ${labels.madeWith}`);
    return `${chunks.join("\n\n")}\n`;
}

export function toMarkdown(book: Book, opts: PlainOptions = {}): string {
    return render(book, "md", opts);
}

export function toText(book: Book, opts: PlainOptions = {}): string {
    return render(book, "text", opts);
}
