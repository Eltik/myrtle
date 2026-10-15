import type { IArchiveOgData } from "../archive";
import { AccentStrip, ArtScrim, BrandRow, CardRoot, emWidth, FG, FG_06, FG_08, FG_55, FG_70, FootRow, Kicker, RARITY_COLOR, siteHost } from "./Frame";

/** The archives' accent: the site's teal, so the lore view never reads as the operator card's rarity orange. */
const TEAL = "#60d7cf";

/** The art's box: the right half of the card. */
const ART_LEFT = 560;
const ART_W = 1200 - ART_LEFT;

function nameFontSize(name: string): number {
    const w = emWidth(name);
    const oneLine = Math.floor(560 / w);
    if (oneLine >= 56) return Math.min(76, oneLine);
    return Math.max(40, Math.min(56, Math.floor(1040 / w)));
}

export function ArchiveTemplate(data: IArchiveOgData) {
    const { name, rarity, number, faction, rows, excerpt, artURL } = data;
    const rarityColor = RARITY_COLOR[rarity] ?? FG_55;
    const fileLine = [number ? `FILE ${number}` : null, faction].filter(Boolean).join("  ·  ");

    return (
        <CardRoot
            glows={[
                { color: "rgba(96,215,207,0.10)", at: "80% 35%" },
                { color: "rgba(236,111,93,0.06)", at: "10% 95%" },
            ]}
        >
            {artURL ? (
                <div style={{ position: "absolute", left: ART_LEFT, top: 0, width: ART_W, height: 630, display: "flex", overflow: "hidden" }}>
                    <img alt="" src={artURL} width={ART_W} height={ART_W} style={{ position: "absolute", left: 0, top: 28, width: ART_W, height: ART_W, objectFit: "cover", objectPosition: "50% 20%" }} />
                </div>
            ) : null}
            <ArtScrim background="linear-gradient(90deg, #0d1216 0%, #0d1216 46%, rgba(13,18,22,0.8) 56%, rgba(13,18,22,0.25) 74%, rgba(13,18,22,0.1) 100%), linear-gradient(0deg, #0d1216 0%, rgba(13,18,22,0.6) 16%, transparent 40%)" />
            {/* A faint ruled grid over the art: the file's paper, and the thing that keeps this card from reading as the operator card. */}
            <div style={{ position: "absolute", left: ART_LEFT, top: 0, width: ART_W, height: 630, display: "flex", background: "repeating-linear-gradient(0deg, rgba(96,215,207,0.05) 0px, rgba(96,215,207,0.05) 1px, transparent 1px, transparent 28px)" }} />
            <div style={{ position: "absolute", left: 64, top: 56, width: 1072, display: "flex" }}>
                <BrandRow kicker="ARCHIVES · DOSSIER" kickerShadow="0 1px 8px rgba(0,0,0,0.6)" />
            </div>
            <div style={{ position: "absolute", left: 64, top: 112, width: 600, height: 436, display: "flex", flexDirection: "column", justifyContent: "center" }}>
                {fileLine ? (
                    <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
                        <div style={{ display: "flex", width: 10, height: 10, borderRadius: 2, background: rarityColor }} />
                        <Kicker size={14} tracking="0.16em" color={TEAL}>
                            {fileLine}
                        </Kicker>
                    </div>
                ) : null}
                <div style={{ display: "flex", marginTop: 14, fontSize: nameFontSize(name), fontWeight: 700, lineHeight: 1, letterSpacing: "-0.028em", color: FG }}>{name}</div>
                {rows.length > 0 ? (
                    <div style={{ display: "flex", flexDirection: "column", marginTop: 24, padding: "14px 18px", gap: 9, width: 520, borderRadius: 12, background: FG_06, border: `1px solid ${FG_08}`, borderLeft: `3px solid ${TEAL}` }}>
                        {rows.map((r) => (
                            <div key={r.label} style={{ display: "flex", alignItems: "baseline", gap: 16 }}>
                                <div style={{ display: "flex", width: 150, fontFamily: "Geist Mono", fontSize: 12, letterSpacing: "0.16em", textTransform: "uppercase", color: FG_55 }}>{r.label}</div>
                                <div style={{ display: "flex", flex: 1, fontFamily: "Geist Mono", fontSize: 17, fontWeight: 500, color: FG }}>{r.value}</div>
                            </div>
                        ))}
                    </div>
                ) : null}
                {excerpt ? <div style={{ display: "flex", marginTop: 20, width: 560, fontSize: 19, lineHeight: 1.45, color: FG_70 }}>{excerpt}</div> : null}
            </div>
            <FootRow path={`${siteHost()} / archives`} />
            <AccentStrip color={TEAL} />
        </CardRoot>
    );
}
