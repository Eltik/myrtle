import { formatNumberCompact } from "#/lib/utils";
import { AccentStrip, BG, BrandMark, FG, FG_06, FG_08, FG_45, FG_55, FG_70, FootRow } from "./Frame";

export interface ITierListOperatorPreview {
    id: string;
    name: string;
    avatarURL?: string;
    rarity: number;
}

export interface ITierListTierPreview {
    name: string;
    color: string;
    operators: ITierListOperatorPreview[];
    operatorCount: number;
}

export interface ITierListOgData {
    title: string;
    slug: string;
    description?: string;
    listType: "official" | "community";
    flairLabel?: string;
    flairColor?: string;
    authorName?: string;
    authorAvatarURL?: string;
    views?: number;
    favorites?: number;
    isTrending?: boolean;
    updatedRelative?: string;
    totalOperators: number;
    tierCount: number;
    tiers: ITierListTierPreview[];
}

const DEFAULT_ACCENT = "#ec6f5d";

function titleFontSize(name: string): number {
    const len = name.length;
    if (len <= 14) return 76;
    if (len <= 22) return 60;
    if (len <= 34) return 48;
    if (len <= 48) return 40;
    return 32;
}

function truncate(text: string, max: number): string {
    if (text.length <= max) return text;
    return `${text.slice(0, max - 1).trimEnd()}…`;
}

function tierGlyph(name: string): string {
    const trimmed = name.trim();
    if (trimmed.length <= 2) return trimmed;
    const words = trimmed.split(/\s+/);
    if (words.length > 1)
        return words
            .map((w) => w[0])
            .join("")
            .slice(0, 2)
            .toUpperCase();
    return trimmed.slice(0, 2);
}

function mixHex(color: string, target: string, amount: number): string {
    const a = parseHex(color);
    const b = parseHex(target);
    if (!a || !b) return color;
    const m = (x: number, y: number) => Math.round(x + (y - x) * amount);
    const r = m(a.r, b.r).toString(16).padStart(2, "0");
    const g = m(a.g, b.g).toString(16).padStart(2, "0");
    const bl = m(a.b, b.b).toString(16).padStart(2, "0");
    return `#${r}${g}${bl}`;
}

function parseHex(hex: string): { r: number; g: number; b: number } | null {
    const m = hex.match(/^#([0-9a-f]{6})$/i);
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

export function TierListTemplate(data: ITierListOgData) {
    const { title, slug, description, listType, flairLabel, flairColor, authorName, authorAvatarURL, views = 0, favorites = 0, isTrending = false, updatedRelative, totalOperators, tierCount, tiers } = data;

    const accent = flairColor ?? DEFAULT_ACCENT;
    const accentDeep = mixHex(accent, "#0d1216", 0.65);
    const accentTint = mixHex(accent, "#11171c", 0.85);

    return (
        <div
            style={{
                width: 1200,
                height: 630,
                display: "flex",
                flexDirection: "row",
                background: BG,
                color: FG,
                fontFamily: "Inter",
                position: "relative",
            }}
        >
            <HeroPanel title={title} description={description} listType={listType} flairLabel={flairLabel} authorName={authorName} authorAvatarURL={authorAvatarURL} updatedRelative={updatedRelative} isTrending={isTrending} accent={accent} accentDeep={accentDeep} accentTint={accentTint} tiers={tiers} />
            <SidePanel slug={slug} totalOperators={totalOperators} tierCount={tierCount} views={views} favorites={favorites} tiers={tiers} />
            <FootRow path={`myrtle.moe / tier-lists`} />
            <AccentStrip color={accent} />
        </div>
    );
}

interface IHeroPanelProps {
    title: string;
    description?: string;
    listType: "official" | "community";
    flairLabel?: string;
    authorName?: string;
    authorAvatarURL?: string;
    updatedRelative?: string;
    isTrending: boolean;
    accent: string;
    accentDeep: string;
    accentTint: string;
    tiers: ITierListTierPreview[];
}

function HeroPanel({ title, description, listType, flairLabel, authorName, authorAvatarURL, updatedRelative, isTrending, accent, accentDeep, accentTint, tiers }: IHeroPanelProps) {
    const decorTiers = tiers.slice(0, 4);

    return (
        <div
            style={{
                width: 660,
                height: 630,
                display: "flex",
                position: "relative",
                background: `linear-gradient(135deg, ${accentTint} 0%, ${accentDeep} 70%, #0d1216 100%)`,
                overflow: "hidden",
            }}
        >
            <div
                style={{
                    position: "absolute",
                    top: 0,
                    left: 0,
                    width: 660,
                    height: 630,
                    display: "flex",
                    background: `radial-gradient(ellipse 70% 60% at 80% 25%, ${accent}33, transparent 65%)`,
                }}
            />
            <TierStack tiers={decorTiers} />
            <div
                style={{
                    position: "absolute",
                    left: 0,
                    bottom: 0,
                    width: 660,
                    height: 380,
                    display: "flex",
                    background: "linear-gradient(180deg, transparent 0%, rgba(13,18,22,0.85) 60%, #0d1216 100%)",
                }}
            />
            <div
                style={{
                    position: "absolute",
                    top: 0,
                    bottom: 0,
                    right: 0,
                    width: 80,
                    display: "flex",
                    background: "linear-gradient(90deg, transparent, #0d1216 80%)",
                }}
            />
            <div
                style={{
                    position: "absolute",
                    left: 56,
                    bottom: 72,
                    display: "flex",
                    flexDirection: "column",
                    maxWidth: 560,
                }}
            >
                <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 18 }}>
                    <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: 13, letterSpacing: "0.22em", textTransform: "uppercase", color: FG_55 }}>TIER LIST</div>
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
                                background: `${accent}1c`,
                                border: `1px solid ${accent}66`,
                                color: accent,
                                fontFamily: "Geist Mono",
                                fontWeight: 700,
                                fontSize: 10,
                                letterSpacing: "0.18em",
                                textTransform: "uppercase",
                                lineHeight: 1,
                            }}
                        >
                            <div style={{ display: "flex", width: 6, height: 6, borderRadius: 999, background: accent }} />
                            {flairLabel}
                        </div>
                    ) : null}
                    {isTrending ? (
                        <div
                            style={{
                                display: "flex",
                                alignItems: "center",
                                gap: 5,
                                padding: "3px 10px",
                                borderRadius: 999,
                                background: "rgba(236,111,93,0.10)",
                                border: "1px solid rgba(236,111,93,0.45)",
                                color: "#ff8a78",
                                fontFamily: "Geist Mono",
                                fontWeight: 700,
                                fontSize: 10,
                                letterSpacing: "0.18em",
                                textTransform: "uppercase",
                                lineHeight: 1,
                            }}
                        >
                            Trending
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
                        maxWidth: 540,
                    }}
                >
                    {truncate(title, 80)}
                </div>
                {description ? (
                    <div
                        style={{
                            display: "flex",
                            fontSize: 17,
                            fontWeight: 400,
                            lineHeight: 1.45,
                            color: "rgba(247,249,251,0.82)",
                            marginTop: 22,
                            maxWidth: 520,
                        }}
                    >
                        {truncate(description, 130)}
                    </div>
                ) : null}
                <div
                    style={{
                        display: "flex",
                        alignItems: "center",
                        gap: 12,
                        marginTop: 26,
                        fontFamily: "Geist Mono",
                        fontSize: 14,
                        color: FG_70,
                        letterSpacing: "0.04em",
                    }}
                >
                    <div style={{ display: "flex", alignItems: "center", gap: 9 }}>
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
                                fontFamily: "Inter",
                            }}
                        >
                            {authorAvatarURL ? <img alt="" src={authorAvatarURL} width={26} height={26} style={{ width: 26, height: 26, objectFit: "cover" }} /> : <div style={{ display: "flex", fontSize: 12, fontWeight: 700, color: FG, lineHeight: 1 }}>{(authorName?.charAt(0) || "?").toUpperCase()}</div>}
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
        </div>
    );
}

const TIER_BADGE_SIZE = 128;

function TierStack({ tiers }: { tiers: ITierListTierPreview[] }) {
    if (tiers.length === 0) return null;
    return (
        <div
            style={{
                position: "absolute",
                top: 56,
                right: -32,
                display: "flex",
                flexDirection: "column",
                gap: 16,
                transform: "rotate(-4deg)",
                opacity: 0.96,
                transformOrigin: "center",
            }}
        >
            {tiers.map((tier, i) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: stack order is a stable positional slot
                <TierStackBadge key={i} tier={tier} />
            ))}
        </div>
    );
}

function TierStackBadge({ tier }: { tier: ITierListTierPreview }) {
    const text = readableOn(tier.color);
    return (
        <div
            style={{
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                width: TIER_BADGE_SIZE,
                height: TIER_BADGE_SIZE,
                borderRadius: 22,
                background: tier.color,
                color: text,
                fontFamily: "Inter",
                fontWeight: 800,
                fontSize: Math.round(TIER_BADGE_SIZE * 0.55),
                letterSpacing: "-0.04em",
                lineHeight: 1,
                boxShadow: `0 18px 36px rgba(0,0,0,0.35), 0 0 0 1px ${tier.color}55`,
            }}
        >
            {tierGlyph(tier.name)}
        </div>
    );
}

interface ISidePanelProps {
    slug: string;
    totalOperators: number;
    tierCount: number;
    views: number;
    favorites: number;
    tiers: ITierListTierPreview[];
}

function SidePanel({ slug, totalOperators, tierCount, views, favorites, tiers }: ISidePanelProps) {
    const top = tiers.slice(0, 3);
    const truncatedSlug = slug.length > 18 ? `${slug.slice(0, 17)}…` : slug;

    return (
        <div
            style={{
                width: 540,
                height: 630,
                display: "flex",
                flexDirection: "column",
                padding: "44px 56px 60px 28px",
                background: BG,
                gap: 22,
            }}
        >
            <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
                    <BrandMark />
                    <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: 14, letterSpacing: "0.22em", textTransform: "uppercase", fontWeight: 600, color: "#d7dbe0" }}>MYRTLE</div>
                </div>
                <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: 11, letterSpacing: "0.18em", textTransform: "uppercase", color: FG_45 }}>
                    SLUG&nbsp;·&nbsp;<span style={{ color: FG_70, marginLeft: 4 }}>{truncatedSlug}</span>
                </div>
            </div>
            <div
                style={{
                    display: "flex",
                    border: `1px solid ${FG_08}`,
                    borderRadius: 12,
                    overflow: "hidden",
                    background: "rgba(255,255,255,0.07)",
                    gap: 1,
                }}
            >
                {[
                    { k: "Tiers", v: String(tierCount), red: false },
                    { k: "Operators", v: String(totalOperators), red: true },
                    { k: "Views", v: formatNumberCompact(views), red: false },
                    { k: "Favs", v: formatNumberCompact(favorites), red: false },
                ].map((s) => (
                    <div
                        key={s.k}
                        style={{
                            flex: 1,
                            display: "flex",
                            flexDirection: "column",
                            gap: 5,
                            background: BG,
                            padding: "12px 14px 14px",
                        }}
                    >
                        <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: 9.5, letterSpacing: "0.22em", textTransform: "uppercase", color: FG_55 }}>{s.k}</div>
                        <div
                            style={{
                                display: "flex",
                                fontSize: 26,
                                fontWeight: 700,
                                lineHeight: 1,
                                letterSpacing: "-0.02em",
                                color: s.red ? "#ff8a78" : FG,
                            }}
                        >
                            {s.v}
                        </div>
                    </div>
                ))}
            </div>
            {top.length > 0 ? (
                <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                    <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
                        <div style={{ display: "flex", height: 1, width: 16, background: FG_45 }} />
                        <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: 10, letterSpacing: "0.24em", textTransform: "uppercase", color: FG_70 }}>Top Tiers</div>
                    </div>
                    <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
                        {top.map((tier, i) => (
                            // biome-ignore lint/suspicious/noArrayIndexKey: top-tier position is a stable display slot
                            <TierRow key={i} tier={tier} />
                        ))}
                    </div>
                </div>
            ) : null}
        </div>
    );
}

function TierRow({ tier }: { tier: ITierListTierPreview }) {
    const text = readableOn(tier.color);
    const ops = tier.operators.slice(0, 5);
    const overflow = tier.operatorCount - ops.length;

    return (
        <div
            style={{
                display: "flex",
                gap: 10,
                background: FG_06,
                border: `1px solid ${FG_08}`,
                borderRadius: 10,
                borderLeft: `3px solid ${tier.color}`,
                padding: "8px 10px",
                alignItems: "center",
            }}
        >
            <div
                style={{
                    display: "flex",
                    width: 44,
                    height: 44,
                    borderRadius: 8,
                    background: tier.color,
                    color: text,
                    alignItems: "center",
                    justifyContent: "center",
                    fontFamily: "Inter",
                    fontWeight: 800,
                    fontSize: 22,
                    letterSpacing: "-0.02em",
                    lineHeight: 1,
                    flexShrink: 0,
                }}
            >
                {tierGlyph(tier.name)}
            </div>
            <div style={{ display: "flex", flexDirection: "column", gap: 3, width: 86, flexShrink: 0 }}>
                <div style={{ display: "flex", fontSize: 13, fontWeight: 700, color: FG, lineHeight: 1.05, letterSpacing: "-0.01em" }}>{truncate(tier.name, 12)}</div>
                <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: 10, letterSpacing: "0.06em", color: FG_55 }}>{tier.operatorCount} ops</div>
            </div>
            <div style={{ display: "flex", alignItems: "center", gap: 0, flexShrink: 0 }}>
                {ops.map((op, i) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: avatar overlap order is a stable slot
                    <AvatarChip key={i} avatarURL={op.avatarURL} index={i} />
                ))}
                {overflow > 0 ? (
                    <div
                        style={{
                            display: "flex",
                            height: 36,
                            paddingLeft: 8,
                            paddingRight: 8,
                            marginLeft: -10,
                            borderRadius: 999,
                            background: FG_06,
                            border: `2px solid #0d1216`,
                            alignItems: "center",
                            justifyContent: "center",
                            fontFamily: "Geist Mono",
                            fontSize: 10,
                            fontWeight: 700,
                            color: FG_70,
                        }}
                    >
                        +{overflow}
                    </div>
                ) : null}
            </div>
        </div>
    );
}

function AvatarChip({ avatarURL, index }: { avatarURL?: string; index: number }) {
    return (
        <div
            style={{
                display: "flex",
                width: 36,
                height: 36,
                borderRadius: 999,
                background: BG,
                border: `2px solid #0d1216`,
                marginLeft: index === 0 ? 0 : -10,
                overflow: "hidden",
                alignItems: "center",
                justifyContent: "center",
            }}
        >
            {avatarURL ? <img alt="" src={avatarURL} width={32} height={32} style={{ width: 32, height: 32, objectFit: "cover", borderRadius: 999 }} /> : null}
        </div>
    );
}
