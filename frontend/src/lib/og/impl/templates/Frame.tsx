/**
 * Shared atoms for myrtle.moe OG templates. All elements are Satori-compatible
 * (every layout container is `display: flex` and there are no className refs).
 */

import type { CSSProperties, ReactNode } from "react";
import { OG_CONFIG } from "../config";

export { RARITY_HEX as RARITY_COLOR } from "#/lib/utils";

/** Strip protocol/trailing slash for the footer-style display label. */
export function siteHost(): string {
    return OG_CONFIG.siteURL.replace(/^https?:\/\//, "").replace(/\/$/, "");
}

export const FG = "#f7f9fb";
export const FG_70 = "rgba(247,249,251,0.70)";
export const FG_55 = "rgba(247,249,251,0.55)";
export const FG_45 = "rgba(247,249,251,0.45)";
export const FG_06 = "rgba(255,255,255,0.06)";
export const FG_08 = "rgba(255,255,255,0.08)";
export const BG = "#0d1216";

export function BrandMark({ size = 28 }: { size?: number }) {
    return (
        <div style={{ display: "flex", width: size, height: size }}>
            <img alt="" src={`${OG_CONFIG.siteURL.replace(/\/$/, "")}/logo/bust_transparent.png`} width={size} height={size} style={{ width: size, height: size, objectFit: "contain" }} />
        </div>
    );
}

/** The header: the brand mark and wordmark, then `kicker` at the far end. `kickerShadow` (a CSS text-shadow) keeps the kicker legible where it lands on art. */
export function BrandRow({ kicker = "ARKNIGHTS · COMPANION", kickerShadow }: { kicker?: string; kickerShadow?: string }) {
    return (
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", width: "100%" }}>
            <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
                <BrandMark />
                <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: 16, letterSpacing: "0.22em", textTransform: "uppercase", fontWeight: 600, color: "#d7dbe0" }}>MYRTLE</div>
            </div>
            <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: 13, letterSpacing: "0.2em", textTransform: "uppercase", color: FG_55, ...(kickerShadow ? { textShadow: kickerShadow } : {}) }}>{kicker}</div>
        </div>
    );
}

export function FootRow({ path = siteHost(), version = `v${OG_CONFIG.designVersion}` }: { path?: string; version?: string }) {
    return (
        <div
            style={{
                position: "absolute",
                left: 64,
                right: 64,
                bottom: 32,
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
                fontFamily: "Geist Mono",
                fontSize: 14,
                letterSpacing: "0.1em",
                color: FG_45,
            }}
        >
            <div style={{ display: "flex" }}>{path}</div>
            <div
                style={{
                    display: "flex",
                    padding: "3px 9px",
                    border: `1px solid ${FG_08}`,
                    borderRadius: 6,
                    fontSize: 11,
                }}
            >
                {version}
            </div>
        </div>
    );
}

/** Bottom accent: one colour, transparent -> solid -> transparent. */
export function AccentStrip({ color = "#ec6f5d" }: { color?: string } = {}) {
    return (
        <div
            style={{
                position: "absolute",
                left: 0,
                right: 0,
                bottom: 0,
                width: 1200,
                height: 6,
                display: "flex",
                background: `linear-gradient(90deg, ${color}00 0%, ${color} 50%, ${color}00 100%)`,
            }}
        />
    );
}

/** Ink width of `text` in em, roughly: CJK and fullwidth glyphs are a full em, Latin capitals about two thirds, the rest a little over half. */
export function emWidth(text: string): number {
    let w = 0;
    for (const ch of text) {
        const c = ch.codePointAt(0) ?? 0;
        if ((c >= 0x2e80 && c <= 0x9fff) || (c >= 0xac00 && c <= 0xd7af) || (c >= 0xff00 && c <= 0xffef)) w += 1;
        else if (/[A-Z0-9MW@]/.test(ch)) w += 0.66;
        else w += 0.56;
    }
    return w;
}

/** The brand accent: the coral every card falls back to when it has no rarity colour of its own. */
export const CORAL = "#ec6f5d";
/** The secondary glow, a teal a card can wash its background with beside the warm one. */
export const TEAL_GLOW = "rgba(96,215,207,0.10)";
/** The warm wash the operator card sits on. */
export const WARM_GLOW = "rgba(247,164,82,0.10)";

const SLATE_GRADIENT = "linear-gradient(135deg, #0e141a 0%, #15202a 60%, #0c1218 100%)";

/** One radial wash over the slate background: `color` centred at `at` (CSS position). */
export interface ICardGlow {
    color: string;
    at?: string;
}

/**
 * The 1200x630 root every card shares: the dark 135deg slate gradient with
 * zero or more radial washes over it. The operator card is one warm wash at
 * 80% 30%, the default.
 */
export function CardRoot({ glows = [{ color: WARM_GLOW }], children }: { glows?: ICardGlow[]; children?: ReactNode }) {
    const washes = glows.map((g) => `radial-gradient(ellipse 70% 60% at ${g.at ?? "80% 30%"}, ${g.color}, transparent 65%)`);
    return (
        <div
            style={{
                width: 1200,
                height: 630,
                display: "flex",
                background: [...washes, SLATE_GRADIENT].join(", "),
                color: FG,
                fontFamily: "Inter",
                position: "relative",
                overflow: "hidden",
            }}
        >
            {children}
        </div>
    );
}

/** A Geist Mono uppercase tracked label: section titles, stat labels, the class line. */
export function Kicker({ children, size = 12, color = FG_55, tracking = "0.22em", style }: { children?: ReactNode; size?: number; color?: string; tracking?: string; style?: CSSProperties }) {
    return <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: size, letterSpacing: tracking, textTransform: "uppercase", color, ...style }}>{children}</div>;
}

/**
 * A 999-radius pill in the operator card's tag style. Neutral is a hairline on
 * the faint white fill; `tint` (a #rrggbb colour) washes the fill, border and
 * text in that colour instead.
 */
export function Chip({ label, tint }: { label: string; tint?: string }) {
    return (
        <div
            style={{
                display: "flex",
                fontFamily: "Geist Mono",
                fontSize: 13,
                letterSpacing: "0.12em",
                textTransform: "uppercase",
                color: tint ? `${tint}d8` : FG_70,
                background: tint ? `${tint}10` : FG_06,
                border: `1px solid ${tint ? `${tint}55` : FG_08}`,
                padding: "7px 12px",
                borderRadius: 999,
            }}
        >
            {label.toUpperCase()}
        </div>
    );
}

/** A row of chips with the operator card's spacing. */
export function ChipRow({ children, marginTop = 28 }: { children?: ReactNode; marginTop?: number }) {
    return <div style={{ display: "flex", gap: 10, flexWrap: "wrap", marginTop }}>{children}</div>;
}

export interface IStatCell {
    label: string;
    value: string;
    /** The value's colour; FG when absent. */
    color?: string;
}

/** The operator card's stat row, any number of equal cells split by hairlines. */
export function StatStrip({ cells, marginTop = 28 }: { cells: IStatCell[]; marginTop?: number }) {
    return (
        <div
            style={{
                display: "flex",
                marginTop,
                background: FG_06,
                border: `1px solid ${FG_08}`,
                borderRadius: 12,
                overflow: "hidden",
            }}
        >
            {cells.map((s, i) => (
                <div
                    key={s.label}
                    style={{
                        flex: 1,
                        display: "flex",
                        flexDirection: "column",
                        gap: 4,
                        padding: "12px 16px",
                        borderLeft: i === 0 ? "none" : `1px solid ${FG_08}`,
                    }}
                >
                    <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: 10, letterSpacing: "0.22em", textTransform: "uppercase", color: FG_55 }}>{s.label}</div>
                    <div style={{ display: "flex", fontSize: 22, fontWeight: 700, color: s.color ?? FG, lineHeight: 1, letterSpacing: "-0.01em" }}>{s.value}</div>
                </div>
            ))}
        </div>
    );
}

/** A single inline-SVG star: Inter has no ★ glyph and the rarity_yellow_N.png assets bake N stars in a row. */
export function StarIcon({ size = 22, color = "#1c1209" }: { size?: number; color?: string }) {
    return (
        // biome-ignore lint/a11y/noSvgWithoutTitle: Satori renders <title> as visible text
        <svg width={String(size)} height={String(size)} viewBox="0 0 24 24" fill={color}>
            <path d="M12 2.5l2.92 5.92 6.53.95-4.72 4.6 1.11 6.51L12 17.5l-5.84 3.07 1.11-6.51-4.72-4.6 6.53-.95L12 2.5z" />
        </svg>
    );
}

/** The solid rarity pill: a star and `label` in dark ink on `color`. */
export function StarPill({ label, color }: { label: string | number; color: string }) {
    return (
        <div
            style={{
                display: "flex",
                alignItems: "center",
                gap: 6,
                padding: "6px 16px 6px 12px",
                borderRadius: 999,
                background: color,
                color: "#1c1209",
                fontWeight: 800,
                fontSize: 24,
                letterSpacing: "0.02em",
                lineHeight: 1,
            }}
        >
            <StarIcon />
            <div style={{ display: "flex" }}>{label}</div>
        </div>
    );
}

/**
 * A full-card scrim over right-hand art, `background` being the gradients to
 * paint. The box is sized explicitly: satori gives an `inset: 0` absolute box
 * no size, so a gradient on one never paints (docs/story-reader.md, the satori
 * trap). The operator card still carries such an overlay and keeps it, since
 * its look is fixed.
 */
export function ArtScrim({ background }: { background: string }) {
    return <div style={{ position: "absolute", left: 0, top: 0, width: 1200, height: 630, display: "flex", background }} />;
}
