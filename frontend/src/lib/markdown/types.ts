export type InlineNode = { type: "text"; value: string } | { type: "strong"; children: InlineNode[] } | { type: "emphasis"; children: InlineNode[] } | { type: "code"; value: string } | { type: "link"; href: string; children: InlineNode[] } | { type: "break" };

export type BlockNode =
    | { type: "heading"; depth: 1 | 2 | 3; children: InlineNode[] }
    | { type: "paragraph"; children: InlineNode[] }
    | { type: "list"; ordered: boolean; start: number; items: BlockNode[][] }
    | { type: "blockquote"; children: BlockNode[] }
    | { type: "code"; lang: string | null; value: string }
    | { type: "hr" };

export type Root = BlockNode[];
