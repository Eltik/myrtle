import type React from "react";
import { useMemo } from "react";
import { cn } from "#/lib/utils";
import { tokenize } from "./tokenize";
import type { BlockNode, InlineNode } from "./types";

interface IMarkdownProps {
    text: string;
    className?: string;
    /** Disables top/bottom margins on the outermost children (useful inside clamps/cards). */
    flush?: boolean;
}

/**
 * Renders sanitized markdown as React elements. No `dangerouslySetInnerHTML`.
 * Supports headings (1-3), paragraphs, bold/italic/inline code, fenced code,
 * lists, blockquotes, hr, and safe links.
 */
export function Markdown({ text, className, flush = false }: IMarkdownProps): React.ReactElement | null {
    const blocks = useMemo(() => tokenize(text ?? ""), [text]);
    if (blocks.length === 0) return null;
    return (
        <div className={cn("markdown-body wrap-break-word", flush && "[&>*:first-child]:mt-0 [&>*:last-child]:mb-0", className)} data-slot="markdown">
            {blocks.map((b, idx) => renderBlock(b, idx))}
        </div>
    );
}

function renderBlock(block: BlockNode, key: number): React.ReactNode {
    switch (block.type) {
        case "heading":
            return renderHeading(block, key);
        case "paragraph":
            return (
                <p key={key} className="my-2 first:mt-0 last:mb-0">
                    {renderInlines(block.children)}
                </p>
            );
        case "list":
            return renderList(block, key);
        case "blockquote":
            return (
                <blockquote key={key} className="my-2 border-border border-l-2 pl-3 text-muted-foreground italic first:mt-0 last:mb-0">
                    {block.children.map((b, i) => renderBlock(b, i))}
                </blockquote>
            );
        case "code":
            return (
                <pre key={key} className="my-2 overflow-x-auto rounded-md border border-border bg-muted/40 px-3 py-2 font-mono text-[12.5px] leading-relaxed first:mt-0 last:mb-0">
                    <code data-lang={block.lang ?? undefined}>{block.value}</code>
                </pre>
            );
        case "hr":
            return <hr key={key} className="my-3 border-border border-t first:mt-0 last:mb-0" />;
    }
}

function renderHeading(block: Extract<BlockNode, { type: "heading" }>, key: number): React.ReactNode {
    const children = renderInlines(block.children);
    if (block.depth === 1) {
        return (
            <h3 key={key} className="mt-3 mb-1.5 font-semibold text-[15.5px] leading-snug tracking-tight first:mt-0 last:mb-0">
                {children}
            </h3>
        );
    }
    if (block.depth === 2) {
        return (
            <h4 key={key} className="mt-2.5 mb-1.5 font-semibold text-[14px] leading-snug tracking-tight first:mt-0 last:mb-0">
                {children}
            </h4>
        );
    }
    return (
        <h5 key={key} className="mt-2 mb-1 font-semibold text-[13px] uppercase tracking-[0.06em] first:mt-0 last:mb-0">
            {children}
        </h5>
    );
}

function renderList(block: Extract<BlockNode, { type: "list" }>, key: number): React.ReactNode {
    const itemClass = "[&>p]:my-1 [&>p:first-child]:mt-0 [&>p:last-child]:mb-0";
    if (block.ordered) {
        return (
            <ol key={key} start={block.start} className="my-2 list-outside list-decimal space-y-0.5 pl-5 marker:text-muted-foreground first:mt-0 last:mb-0">
                {block.items.map((item, i) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: stateless render - list order is stable per input
                    <li key={i} className={itemClass}>
                        {item.map((b, bi) => renderBlock(b, bi))}
                    </li>
                ))}
            </ol>
        );
    }
    return (
        <ul key={key} className="my-2 list-outside list-disc space-y-0.5 pl-5 marker:text-muted-foreground first:mt-0 last:mb-0">
            {block.items.map((item, i) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: stateless render - list order is stable per input
                <li key={i} className={itemClass}>
                    {item.map((b, bi) => renderBlock(b, bi))}
                </li>
            ))}
        </ul>
    );
}

function renderInlines(nodes: InlineNode[]): React.ReactNode {
    return nodes.map((n, i) => renderInline(n, i));
}

function renderInline(node: InlineNode, key: number): React.ReactNode {
    switch (node.type) {
        case "text":
            return <span key={key}>{node.value}</span>;
        case "strong":
            return (
                <strong key={key} className="font-semibold text-foreground">
                    {renderInlines(node.children)}
                </strong>
            );
        case "emphasis":
            return (
                <em key={key} className="italic">
                    {renderInlines(node.children)}
                </em>
            );
        case "code":
            return (
                <code key={key} className="rounded-sm border border-border bg-muted/50 px-1 py-px font-mono text-[0.9em]">
                    {node.value}
                </code>
            );
        case "link":
            return (
                <a key={key} href={node.href} target="_blank" rel="noreferrer noopener" className="text-primary underline decoration-primary/30 underline-offset-2 hover:decoration-primary">
                    {renderInlines(node.children)}
                </a>
            );
        case "break":
            return <br key={key} />;
    }
}
