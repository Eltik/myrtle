import { tokenize } from "./tokenize";
import type { BlockNode, InlineNode } from "./types";

/**
 * Renders markdown source to a plain-text string, useful for search indexing,
 * line-clamped previews, OG image alt text, and any other context where
 * formatting markers shouldn't leak through.
 */
export function stripMarkdown(src: string): string {
    if (!src) return "";
    const blocks = tokenize(src);
    return blocksToText(blocks).trim();
}

function blocksToText(blocks: BlockNode[]): string {
    const out: string[] = [];
    for (const b of blocks) {
        switch (b.type) {
            case "heading":
            case "paragraph":
                out.push(inlineToText(b.children));
                break;
            case "blockquote":
                out.push(blocksToText(b.children));
                break;
            case "list":
                for (const item of b.items) out.push(blocksToText(item));
                break;
            case "code":
                out.push(b.value);
                break;
            case "hr":
                break;
        }
    }
    return out.join(" ").replace(/\s+/g, " ");
}

function inlineToText(nodes: InlineNode[]): string {
    let out = "";
    for (const n of nodes) {
        switch (n.type) {
            case "text":
                out += n.value;
                break;
            case "code":
                out += n.value;
                break;
            case "strong":
            case "emphasis":
            case "link":
                out += inlineToText(n.children);
                break;
            case "break":
                out += " ";
                break;
        }
    }
    return out;
}
