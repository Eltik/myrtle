import { GRID_TITLE_MAX } from "#/components/grids/shared";
import type { ArtFit } from "#/lib/api/tier-entities";
import type { IGridImageCell, IGridImageData } from "../grid";
import type { IRenderDimensions } from "../render";
import { FG, FG_45, siteHost } from "./Frame";

// The grid board as a PNG: the same dark board the page draws, a bold centred
// title over equal square cells, each art over a two-line label strip.

export const GRID_IMAGE_LAYOUT = {
    width: 1200,
    padding: 40,
    titleHeight: 84,
    titleGapBelow: 24,
    footerHeight: 34,
    gap: 8,
    /** A small grid's cells stop growing here, so a 1 x 1 is a card, not a poster. */
    maxCell: 240,
} as const;

const L = GRID_IMAGE_LAYOUT;

const BOARD_BG = "#111114";
const CELL_BG = "#1f1f24";
const STRIP_BG = "#18181c";
const EMPTY_BG = "#3a3a42";

/** A label is drawn on at most this many lines, each this many ems tall. */
const LABEL_LINES = 2;
const LABEL_LINE_HEIGHT = 1.2;

function cellSize(cols: number): number {
    const available = L.width - L.padding * 2 - L.gap * (cols - 1);
    return Math.min(L.maxCell, Math.floor(available / cols));
}

function labelFontSize(cols: number): number {
    if (cols <= 3) return 24;
    if (cols <= 5) return 18;
    if (cols <= 7) return 14;
    return 12;
}

function labelBlockHeight(fontSize: number): number {
    return fontSize * LABEL_LINE_HEIGHT * LABEL_LINES;
}

function stripHeight(cols: number): number {
    return Math.round(labelBlockHeight(labelFontSize(cols)) + 14);
}

export function gridImageDimensions(data: IGridImageData): IRenderDimensions {
    const cell = cellSize(data.cols);
    const board = data.rows * (cell + stripHeight(data.cols)) + (data.rows - 1) * L.gap;
    return { width: L.width, height: L.padding + L.titleHeight + L.titleGapBelow + board + L.footerHeight + L.padding };
}

function titleFontSize(title: string): number {
    const len = title.length;
    if (len <= 24) return 52;
    if (len <= 40) return 42;
    if (len <= 64) return 34;
    return 28;
}

function truncate(text: string, max: number): string {
    const points = Array.from(text);
    if (points.length <= max) return text;
    return `${points
        .slice(0, max - 1)
        .join("")
        .trimEnd()}…`;
}

function initials(name: string): string {
    const words = name.split(/\s+/).filter(Boolean);
    return ((words[0]?.charAt(0) ?? "?") + (words[1]?.charAt(0) ?? "")).toUpperCase();
}

const FIT_INSET: Record<ArtFit, number> = { cover: 0, object: 0.06, glyph: 0.16 };

export function GridImageTemplate(data: IGridImageData) {
    const { width, height } = gridImageDimensions(data);
    const cell = cellSize(data.cols);
    const boardWidth = data.cols * cell + (data.cols - 1) * L.gap;
    const rows = Array.from({ length: data.rows }, (_, r) => data.cells.slice(r * data.cols, (r + 1) * data.cols));

    return (
        <div style={{ width, height, display: "flex", flexDirection: "column", alignItems: "center", background: BOARD_BG, color: FG, fontFamily: "Inter", padding: L.padding }}>
            <div
                style={{
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    height: L.titleHeight,
                    marginBottom: L.titleGapBelow,
                    width: width - L.padding * 2,
                    fontWeight: 800,
                    fontSize: titleFontSize(data.title),
                    letterSpacing: "-0.02em",
                    textAlign: "center",
                }}
            >
                {truncate(data.title, GRID_TITLE_MAX)}
            </div>
            <div style={{ display: "flex", flexDirection: "column", gap: L.gap, width: boardWidth }}>
                {rows.map((row, r) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: a row IS its position
                    <div key={r} style={{ display: "flex", gap: L.gap }}>
                        {row.map((c, i) => (
                            // biome-ignore lint/suspicious/noArrayIndexKey: a cell IS its position
                            <Cell key={i} cell={c} size={cell} cols={data.cols} />
                        ))}
                    </div>
                ))}
            </div>
            <div style={{ display: "flex", flex: 1, alignItems: "flex-end", justifyContent: "flex-end", width: width - L.padding * 2, fontSize: 16, color: FG_45 }}>{siteHost()}</div>
        </div>
    );
}

function Cell({ cell, size, cols }: { cell: IGridImageCell; size: number; cols: number }) {
    const fontSize = labelFontSize(cols);
    const strip = stripHeight(cols);
    // Two lines at roughly 0.56 em per character, the clamp's fallback where satori breaks a word.
    const maxChars = Math.max(6, Math.floor((size - 10) / (fontSize * 0.56)) * LABEL_LINES);
    const inset = Math.round(size * FIT_INSET[cell.fit]);
    const art = size - inset * 2;

    return (
        <div style={{ display: "flex", flexDirection: "column", width: size, borderRadius: 6, overflow: "hidden", background: CELL_BG }}>
            <div style={{ display: "flex", width: size, height: size, alignItems: "center", justifyContent: "center", background: cell.key ? CELL_BG : EMPTY_BG }}>
                {cell.artURL ? (
                    <img alt="" src={cell.artURL} width={art} height={art} style={{ width: art, height: art, objectFit: cell.fit === "cover" ? "cover" : "contain" }} />
                ) : cell.name ? (
                    <div style={{ display: "flex", fontWeight: 700, fontSize: Math.round(size * 0.28), color: FG }}>{initials(cell.name)}</div>
                ) : null}
            </div>
            <div
                style={{
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    width: size,
                    height: strip,
                    padding: "6px 6px",
                    background: STRIP_BG,
                    color: "#ffffff",
                    fontWeight: 700,
                    fontSize,
                    lineHeight: LABEL_LINE_HEIGHT,
                    textAlign: "center",
                    overflow: "hidden",
                }}
            >
                <Label text={truncate(cell.label, maxChars)} fontSize={fontSize} />
            </div>
        </div>
    );
}

/** Inter has no ★ glyph (satori draws a box), so a star in a label ("Favorite 6★") is an inline SVG star. */
function Label({ text, fontSize }: { text: string; fontSize: number }) {
    const maxHeight = labelBlockHeight(fontSize);
    if (!text.includes("★")) return <div style={{ display: "block", lineClamp: 2, overflow: "hidden", maxHeight }}>{text}</div>;
    const parts = text.split("★");
    const star = Math.round(fontSize * 0.95);
    return (
        <div style={{ display: "flex", flexWrap: "wrap", alignItems: "center", justifyContent: "center", overflow: "hidden", maxHeight }}>
            {parts.map((part, i) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: segments of one fixed string
                <div key={i} style={{ display: "flex", alignItems: "center" }}>
                    {part && <span>{part}</span>}
                    {i < parts.length - 1 && (
                        <svg aria-hidden="true" width={star} height={star} viewBox="0 0 24 24" style={{ marginLeft: 1 }}>
                            <path d="m12 2 3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" fill="#ffffff" />
                        </svg>
                    )}
                </div>
            ))}
        </div>
    );
}
