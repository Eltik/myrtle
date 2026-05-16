import type { IRenderDimensions } from "../render";
import { BG, FG, FG_08, FG_55 } from "./Frame";

export interface ITierListBoardImageOperator {
    id: string;
    name: string;
    rarity: number;
    avatarURL?: string;
}

export interface ITierListBoardImageTier {
    name: string;
    color: string;
    operators: ITierListBoardImageOperator[];
}

export interface ITierListBoardImageData {
    title: string;
    slug: string;
    tiers: ITierListBoardImageTier[];
}

export const TIER_LIST_BOARD_IMAGE_LAYOUT = {
    width: 1400,
    sidePadding: 36,
    titleHeight: 88,
    titleGapBelow: 20,
    bottomPadding: 32,
    tierLabelWidth: 168,
    rowPaddingX: 14,
    rowPaddingY: 10,
    rowMinHeight: 92,
    opSize: 84,
    opGap: 6,
} as const;

const L = TIER_LIST_BOARD_IMAGE_LAYOUT;

function opsPerRow(): number {
    const contentWidth = L.width - L.sidePadding * 2 - L.tierLabelWidth - L.rowPaddingX * 2;
    return Math.max(1, Math.floor((contentWidth + L.opGap) / (L.opSize + L.opGap)));
}

function tierRowHeight(operatorCount: number): number {
    const ops = Math.max(operatorCount, 1);
    const rows = Math.ceil(ops / opsPerRow());
    const opsBlockHeight = rows * L.opSize + (rows - 1) * L.opGap;
    return Math.max(L.rowMinHeight, opsBlockHeight + L.rowPaddingY * 2);
}

export function tierListBoardImageDimensions(data: ITierListBoardImageData): IRenderDimensions {
    const board = data.tiers.reduce((sum, t) => sum + tierRowHeight(t.operators.length), 0);
    const height = L.sidePadding + L.titleHeight + L.titleGapBelow + board + L.bottomPadding;
    return { width: L.width, height };
}

function parseHex(hex: string): { r: number; g: number; b: number } | null {
    const m = hex.match(/^#([0-9a-fA-F]{6})$/);
    if (!m) return null;
    const h = m[1] as string;
    return {
        r: Number.parseInt(h.slice(0, 2), 16),
        g: Number.parseInt(h.slice(2, 4), 16),
        b: Number.parseInt(h.slice(4, 6), 16),
    };
}

function readableOn(hex: string): "white" | "black" {
    const c = parseHex(hex);
    if (!c) return "white";
    const lum = (0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b) / 255;
    return lum > 0.62 ? "black" : "white";
}

function titleFontSize(name: string): number {
    const len = name.length;
    if (len <= 22) return 52;
    if (len <= 36) return 42;
    if (len <= 52) return 34;
    return 28;
}

function tierLabelFontSize(name: string): number {
    const total = name.trim().length;
    const longest = name
        .trim()
        .split(/\s+/)
        .reduce((m, w) => Math.max(m, w.length), 0);
    if (total <= 2) return 60;
    if (total <= 6 && longest <= 6) return 30;
    if (total <= 12 && longest <= 8) return 22;
    if (total <= 20 && longest <= 10) return 17;
    return 14;
}

function truncate(text: string, max: number): string {
    if (text.length <= max) return text;
    return `${text.slice(0, max - 1).trimEnd()}…`;
}

const RARITY_BAR_COLOR: Record<number, string> = {
    6: "#f7a452",
    5: "#f7e79e",
    4: "#bcabdb",
    3: "#88c8e3",
    2: "#7ef2a3",
    1: "#b5b5b5",
};

export function TierListBoardImageTemplate(data: ITierListBoardImageData) {
    const { width, height } = tierListBoardImageDimensions(data);

    return (
        <div
            style={{
                width,
                height,
                display: "flex",
                flexDirection: "column",
                background: BG,
                color: FG,
                fontFamily: "Inter",
                padding: `${L.sidePadding}px ${L.sidePadding}px ${L.bottomPadding}px`,
            }}
        >
            <Title title={data.title} />
            <Board tiers={data.tiers} />
        </div>
    );
}

function Title({ title }: { title: string }) {
    return (
        <div
            style={{
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                height: L.titleHeight,
                marginBottom: L.titleGapBelow,
                fontFamily: "Inter",
                fontWeight: 700,
                fontSize: titleFontSize(title),
                letterSpacing: "-0.02em",
                color: FG,
                textAlign: "center",
            }}
        >
            {truncate(title, 96)}
        </div>
    );
}

function Board({ tiers }: { tiers: ITierListBoardImageTier[] }) {
    if (tiers.length === 0) {
        return (
            <div
                style={{
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    flex: 1,
                    border: `1px dashed ${FG_08}`,
                    borderRadius: 14,
                    color: FG_55,
                    fontFamily: "Inter",
                    fontSize: 16,
                }}
            >
                This tier list has no tiers yet.
            </div>
        );
    }

    return (
        <div
            style={{
                display: "flex",
                flexDirection: "column",
                borderRadius: 14,
                overflow: "hidden",
                border: `1px solid ${FG_08}`,
                background: "rgba(255,255,255,0.025)",
            }}
        >
            {tiers.map((tier, i) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: tier order is a stable positional slot
                <TierRow key={i} tier={tier} />
            ))}
        </div>
    );
}

function TierRow({ tier }: { tier: ITierListBoardImageTier }) {
    const textColor = readableOn(tier.color);
    const height = tierRowHeight(tier.operators.length);
    return (
        <div
            style={{
                display: "flex",
                minHeight: height,
                borderBottom: `1px solid ${FG_08}`,
                background: `linear-gradient(to right, ${tier.color}1f, transparent 60%)`,
            }}
        >
            <TierLabel name={tier.name} color={tier.color} textColor={textColor} height={height} />
            {tier.operators.length === 0 ? (
                <div
                    style={{
                        display: "flex",
                        alignItems: "center",
                        flex: 1,
                        padding: `${L.rowPaddingY}px ${L.rowPaddingX}px`,
                        fontFamily: "Inter",
                        fontStyle: "italic",
                        fontSize: 14,
                        color: FG_55,
                    }}
                >
                    No operators in this tier.
                </div>
            ) : (
                <div
                    style={{
                        display: "flex",
                        flexWrap: "wrap",
                        gap: L.opGap,
                        padding: `${L.rowPaddingY}px ${L.rowPaddingX}px`,
                        alignContent: "flex-start",
                        flex: 1,
                    }}
                >
                    {tier.operators.map((op) => (
                        <OperatorTile key={op.id} op={op} />
                    ))}
                </div>
            )}
        </div>
    );
}

function TierLabel({ name, color, textColor, height }: { name: string; color: string; textColor: "white" | "black"; height: number }) {
    return (
        <div
            style={{
                display: "flex",
                width: L.tierLabelWidth,
                minHeight: height,
                background: color,
                color: textColor,
                alignItems: "center",
                justifyContent: "center",
                padding: 12,
                flexShrink: 0,
            }}
        >
            <div
                style={{
                    display: "flex",
                    textAlign: "center",
                    fontFamily: "Inter",
                    fontWeight: 800,
                    fontSize: tierLabelFontSize(name),
                    lineHeight: 1.05,
                    letterSpacing: name.trim().length <= 2 ? "-0.01em" : "-0.005em",
                    maxWidth: L.tierLabelWidth - 16,
                    wordBreak: "break-word",
                }}
            >
                {truncate(name, 60)}
            </div>
        </div>
    );
}

function OperatorTile({ op }: { op: ITierListBoardImageOperator }) {
    const rarityColor = RARITY_BAR_COLOR[op.rarity] ?? RARITY_BAR_COLOR[1];
    return (
        <div
            style={{
                display: "flex",
                width: L.opSize,
                height: L.opSize,
                borderRadius: 10,
                background: "#1d242b",
                overflow: "hidden",
                position: "relative",
                border: `1px solid ${FG_08}`,
            }}
        >
            {op.avatarURL ? (
                <img alt="" src={op.avatarURL} width={L.opSize} height={L.opSize} style={{ width: L.opSize, height: L.opSize, objectFit: "cover" }} />
            ) : (
                <div
                    style={{
                        display: "flex",
                        width: L.opSize,
                        height: L.opSize,
                        alignItems: "center",
                        justifyContent: "center",
                        fontFamily: "Inter",
                        fontWeight: 700,
                        fontSize: 20,
                        color: FG,
                    }}
                >
                    {(op.name.charAt(0) || "?").toUpperCase()}
                </div>
            )}
            <div
                style={{
                    position: "absolute",
                    display: "flex",
                    left: 0,
                    right: 0,
                    bottom: 0,
                    height: 4,
                    background: rarityColor,
                }}
            />
        </div>
    );
}
