import { formatNumberCompact } from "#/lib/utils";
import type { IRenderDimensions } from "../render";
import { BG, FG, FG_06, FG_08, FG_45, FG_55, FG_70 } from "./Frame";

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
    description?: string;
    listType: "official" | "community";
    flairLabel?: string;
    flairColor?: string;
    authorName?: string;
    authorAvatarURL?: string;
    updatedRelative?: string;
    views?: number;
    favorites?: number;
    totalOperators: number;
    tierCount: number;
    tiers: ITierListBoardImageTier[];
}

export const TIER_LIST_BOARD_IMAGE_LAYOUT = {
    width: 1400,
    sidePadding: 36,
    headerBaseHeight: 168,
    headerDescriptionLineHeight: 24,
    headerBottomGap: 36,
    footerHeight: 88,
    tierLabelWidth: 168,
    rowPaddingX: 14,
    rowPaddingY: 10,
    rowMinHeight: 92,
    opSize: 84,
    opGap: 6,
    tierGap: 0,
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

// Description sits in a ~880px-wide column at 16px font. Estimate wrap lines
// generously (≈70 chars per line) and cap at 3 since we truncate to 140 chars.
function headerHeight(description?: string): number {
    if (!description) return L.headerBaseHeight + L.headerBottomGap;
    const lines = Math.min(3, Math.max(1, Math.ceil(description.length / 70)));
    return L.headerBaseHeight + lines * L.headerDescriptionLineHeight + L.headerBottomGap;
}

export function tierListBoardImageDimensions(data: ITierListBoardImageData): IRenderDimensions {
    const board = data.tiers.reduce((sum, t) => sum + tierRowHeight(t.operators.length), 0);
    const gaps = Math.max(0, data.tiers.length - 1) * L.tierGap;
    const height = headerHeight(data.description) + board + gaps + L.footerHeight + L.sidePadding * 2;
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
    if (len <= 22) return 56;
    if (len <= 36) return 46;
    if (len <= 52) return 38;
    return 32;
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
                padding: L.sidePadding,
            }}
        >
            <Header data={data} />
            <Board tiers={data.tiers} />
            <Footer slug={data.slug} />
        </div>
    );
}

function Header({ data }: { data: ITierListBoardImageData }) {
    const { title, description, listType, flairLabel, flairColor, authorName, authorAvatarURL, updatedRelative, views = 0, favorites = 0, totalOperators, tierCount } = data;
    return (
        <div
            style={{
                display: "flex",
                flexDirection: "row",
                alignItems: "flex-start",
                gap: 24,
                paddingBottom: 18,
                borderBottom: `1px solid ${FG_08}`,
                marginBottom: 18,
            }}
        >
            <div style={{ display: "flex", flexDirection: "column", flexGrow: 1, minWidth: 0 }}>
                <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
                    <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: 12, letterSpacing: "0.22em", textTransform: "uppercase", color: FG_55 }}>TIER LIST</div>
                    <div style={{ display: "flex", width: 4, height: 4, borderRadius: 999, background: FG_45 }} />
                    {listType === "official" ? (
                        <div
                            style={{
                                display: "flex",
                                padding: "3px 9px",
                                borderRadius: 999,
                                background: "#f1c84a",
                                color: "#3a2a06",
                                fontFamily: "Inter",
                                fontWeight: 700,
                                fontSize: 11,
                                letterSpacing: "0.14em",
                                textTransform: "uppercase",
                                lineHeight: 1,
                            }}
                        >
                            Official
                        </div>
                    ) : (
                        <div
                            style={{
                                display: "flex",
                                padding: "3px 9px",
                                borderRadius: 999,
                                background: FG_06,
                                border: `1px solid ${FG_08}`,
                                color: FG_70,
                                fontFamily: "Geist Mono",
                                fontWeight: 600,
                                fontSize: 10,
                                letterSpacing: "0.18em",
                                textTransform: "uppercase",
                                lineHeight: 1,
                            }}
                        >
                            Community
                        </div>
                    )}
                    {flairLabel ? (
                        <div
                            style={{
                                display: "flex",
                                alignItems: "center",
                                gap: 6,
                                padding: "3px 10px 3px 8px",
                                borderRadius: 999,
                                background: `${flairColor ?? "#ec6f5d"}1c`,
                                border: `1px solid ${flairColor ?? "#ec6f5d"}66`,
                                color: flairColor ?? "#ff8a78",
                                fontFamily: "Geist Mono",
                                fontWeight: 700,
                                fontSize: 10,
                                letterSpacing: "0.18em",
                                textTransform: "uppercase",
                                lineHeight: 1,
                            }}
                        >
                            <div style={{ display: "flex", width: 6, height: 6, borderRadius: 999, background: flairColor ?? "#ec6f5d" }} />
                            {flairLabel}
                        </div>
                    ) : null}
                </div>

                <div
                    style={{
                        display: "flex",
                        fontSize: titleFontSize(title),
                        fontWeight: 700,
                        lineHeight: 1.05,
                        letterSpacing: "-0.025em",
                        color: FG,
                        marginTop: 14,
                    }}
                >
                    {truncate(title, 96)}
                </div>

                {description ? (
                    <div
                        style={{
                            display: "flex",
                            fontSize: 16,
                            lineHeight: 1.4,
                            color: "rgba(247,249,251,0.78)",
                            marginTop: 10,
                            maxWidth: 880,
                        }}
                    >
                        {truncate(description, 140)}
                    </div>
                ) : null}

                <div
                    style={{
                        display: "flex",
                        alignItems: "center",
                        gap: 10,
                        marginTop: 16,
                        fontFamily: "Geist Mono",
                        fontSize: 13,
                        color: FG_70,
                    }}
                >
                    <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                        <div
                            style={{
                                display: "flex",
                                width: 26,
                                height: 26,
                                borderRadius: 999,
                                overflow: "hidden",
                                background: "rgba(255,255,255,0.10)",
                                border: "1px solid rgba(255,255,255,0.18)",
                                alignItems: "center",
                                justifyContent: "center",
                            }}
                        >
                            {authorAvatarURL ? (
                                <img alt="" src={authorAvatarURL} width={26} height={26} style={{ width: 26, height: 26, objectFit: "cover" }} />
                            ) : (
                                <div style={{ display: "flex", fontFamily: "Inter", fontSize: 12, fontWeight: 700, color: FG, lineHeight: 1 }}>{(authorName?.charAt(0) || "?").toUpperCase()}</div>
                            )}
                        </div>
                        <div style={{ display: "flex", color: FG, fontFamily: "Inter", fontWeight: 600, fontSize: 15, letterSpacing: "-0.005em" }}>{authorName ?? "Community"}</div>
                    </div>
                    {updatedRelative ? (
                        <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
                            <div style={{ display: "flex", fontSize: 18, lineHeight: 1, color: FG_45, fontFamily: "Inter" }}>·</div>
                            <div style={{ display: "flex" }}>{updatedRelative}</div>
                        </div>
                    ) : null}
                </div>
            </div>

            <div style={{ display: "flex", flexShrink: 0, gap: 10, marginTop: 4 }}>
                <StatPill label="Tiers" value={String(tierCount)} />
                <StatPill label="Operators" value={String(totalOperators)} accent />
                <StatPill label="Views" value={formatNumberCompact(views)} />
                <StatPill label="Favs" value={formatNumberCompact(favorites)} />
            </div>
        </div>
    );
}

function StatPill({ label, value, accent = false }: { label: string; value: string; accent?: boolean }) {
    return (
        <div
            style={{
                display: "flex",
                flexDirection: "column",
                gap: 3,
                padding: "8px 14px",
                background: FG_06,
                border: `1px solid ${FG_08}`,
                borderRadius: 10,
                minWidth: 76,
            }}
        >
            <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: 9.5, letterSpacing: "0.22em", textTransform: "uppercase", color: FG_55 }}>{label}</div>
            <div
                style={{
                    display: "flex",
                    fontFamily: "Inter",
                    fontSize: 20,
                    fontWeight: 700,
                    lineHeight: 1,
                    letterSpacing: "-0.02em",
                    color: accent ? "#ff8a78" : FG,
                }}
            >
                {value}
            </div>
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
                gap: L.tierGap,
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

function Footer({ slug }: { slug: string }) {
    return (
        <div
            style={{
                display: "flex",
                alignItems: "center",
                justifyContent: "space-between",
                height: L.footerHeight,
                marginTop: 18,
                paddingTop: 16,
                borderTop: `1px solid ${FG_08}`,
                fontFamily: "Geist Mono",
                fontSize: 14,
                letterSpacing: "0.12em",
                color: FG_55,
                textTransform: "uppercase",
            }}
        >
            <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
                <div style={{ display: "flex", fontFamily: "Inter", fontSize: 18, fontWeight: 700, color: FG, letterSpacing: "-0.01em", textTransform: "none" }}>myrtle.moe</div>
                <div style={{ display: "flex", width: 4, height: 4, borderRadius: 999, background: FG_45 }} />
                <div style={{ display: "flex" }}>tier-lists / {truncate(slug, 64)}</div>
            </div>
            <div
                style={{
                    display: "flex",
                    padding: "5px 10px",
                    border: `1px solid ${FG_08}`,
                    borderRadius: 6,
                    fontSize: 11,
                    color: FG_70,
                }}
            >
                v3
            </div>
        </div>
    );
}
