/**
 * Shared atoms for myrtle.moe OG templates. All elements are Satori-compatible
 * (every layout container is `display: flex` and there are no className refs).
 */

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

/** Bottom accent - single-color gradient that fades transparent → solid → transparent. */
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
