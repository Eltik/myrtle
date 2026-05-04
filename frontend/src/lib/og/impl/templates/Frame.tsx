/**
 * Shared atoms for myrtle.moe OG templates. All elements are Satori-compatible
 * (every layout container is `display: flex` and there are no className refs).
 */

import { OG_CONFIG } from "../config";

/** Strip protocol/trailing slash for the footer-style display label. */
export function siteHost(): string {
    return OG_CONFIG.siteURL.replace(/^https?:\/\//, "").replace(/\/$/, "");
}

export const RARITY_COLOR: Record<number, string> = {
    6: "#f7a452",
    5: "#f7e79e",
    4: "#bcabdb",
    3: "#88c8e3",
    2: "#7ef2a3",
    1: "#ffffff",
};

export const FG = "#f7f9fb";
export const FG_70 = "rgba(247,249,251,0.70)";
export const FG_55 = "rgba(247,249,251,0.55)";
export const FG_45 = "rgba(247,249,251,0.45)";
export const FG_06 = "rgba(255,255,255,0.06)";
export const FG_08 = "rgba(255,255,255,0.08)";
export const BG = "#0d1216";

export function BrandMark({ size = 28 }: { size?: number }) {
    return (
        <div
            style={{
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                width: size,
                height: size,
                borderRadius: 8,
                background: FG_06,
                border: `1px solid ${FG_08}`,
            }}
        >
            {/* biome-ignore lint/a11y/noSvgWithoutTitle: Satori renders <title> as visible text */}
            <svg width={size * 0.64} height={size * 0.64} viewBox="0 0 24 24" fill="none" stroke="#ec6f5d" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M4 18c2.5 0 2.5-4 5-4s2.5 4 5 4 2.5-4 5-4" />
                <path d="M4 12c2.5 0 2.5-4 5-4s2.5 4 5 4 2.5-4 5-4" />
            </svg>
        </div>
    );
}

export function BrandRow({ kicker = "ARKNIGHTS · COMPANION" }: { kicker?: string }) {
    return (
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", width: "100%" }}>
            <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
                <BrandMark />
                <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: 16, letterSpacing: "0.22em", textTransform: "uppercase", fontWeight: 600, color: "#d7dbe0" }}>MYRTLE</div>
            </div>
            <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: 13, letterSpacing: "0.2em", textTransform: "uppercase", color: FG_55 }}>{kicker}</div>
        </div>
    );
}

export function FootRow({ path = siteHost(), version = "v3" }: { path?: string; version?: string }) {
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

export function RainbowStrip() {
    return (
        <div
            style={{
                position: "absolute",
                left: 0,
                right: 0,
                bottom: 0,
                height: 6,
                display: "flex",
                background: `linear-gradient(90deg, ${RARITY_COLOR[1]} 0% 16.66%, ${RARITY_COLOR[2]} 16.66% 33.33%, ${RARITY_COLOR[3]} 33.33% 50%, ${RARITY_COLOR[4]} 50% 66.66%, ${RARITY_COLOR[5]} 66.66% 83.33%, ${RARITY_COLOR[6]} 83.33% 100%)`,
            }}
        />
    );
}
