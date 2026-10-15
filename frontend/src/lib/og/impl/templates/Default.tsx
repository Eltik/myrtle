import { OG_CONFIG } from "../config";
import type { DefaultOgArt } from "../presets";
import { AccentStrip, ArtScrim, BrandRow, CardRoot, emWidth, FG, FG_55, FootRow, Kicker, TEAL_GLOW } from "./Frame";

export interface IDefaultOgData {
    title: string;
    subtitle?: string;
    /** The section's name in the card's locale, drawn as the kicker; absent draws none. */
    section?: string;
    /** Which mascot art the card draws; the key visual when absent. */
    art?: DefaultOgArt;
}

/** The mascot files under `public/home/`. The key visual is 2400x1350; the three chibis are 300 px squares on transparency. */
const ART_FILE: Record<DefaultOgArt, string> = {
    hero: "hero_art.jpg",
    happy: "myrtle_happy.png",
    sleepy: "myrtle_sleepy.png",
    fall: "myrtle_fall.png",
};

function artURL(art: DefaultOgArt): string {
    return `${OG_CONFIG.siteURL.replace(/\/$/, "")}/home/${ART_FILE[art]}`;
}

/** The key visual's box: the right ~62% of the card, full height. */
const HERO_LEFT = 460;
const HERO_W = 1200 - HERO_LEFT;

/** The medallion the chibi sits in, and the chibi's drawn size: 1.4x its 300 px file, a little larger than the disc so the file's own edges fall outside the circle (the fall pose's grey shadow is cut off square by its file). */
const DISC = 400;
const CHIBI = 420;

/** The title's size in the 600 px column: one line up to 88 px where it fits at 64 or more, else two lines sized to fill them. */
function titleFontSize(title: string): number {
    const w = emWidth(title);
    const oneLine = Math.floor(600 / w);
    if (oneLine >= 64) return Math.min(88, oneLine);
    return Math.max(48, Math.min(68, Math.floor(1100 / w)));
}

function HeroArt() {
    return (
        <div style={{ position: "absolute", left: HERO_LEFT, top: 0, width: HERO_W, height: 630, display: "flex", overflow: "hidden" }}>
            {/* The drummer sits left of centre in the 16:9 plate; 42% keeps her and the
                drum in the box. */}
            <img alt="" src={artURL("hero")} width={HERO_W} height={630} style={{ width: HERO_W, height: 630, objectFit: "cover", objectPosition: "42% 30%" }} />
        </div>
    );
}

function ChibiArt({ art }: { art: DefaultOgArt }) {
    return (
        <div style={{ position: "absolute", left: 690, top: 115, width: DISC + 80, height: DISC + 80, display: "flex", alignItems: "center", justifyContent: "center" }}>
            <div
                style={{
                    position: "absolute",
                    left: 0,
                    top: 0,
                    width: DISC + 80,
                    height: DISC + 80,
                    display: "flex",
                    borderRadius: 999,
                    background: "radial-gradient(circle at 50% 50%, rgba(236,111,93,0.20), rgba(96,215,207,0.06) 55%, transparent 70%)",
                }}
            />
            <div style={{ display: "flex", width: DISC, height: DISC, borderRadius: 999, overflow: "hidden", alignItems: "center", justifyContent: "center", background: "rgba(255,255,255,0.05)", border: "1px solid rgba(255,255,255,0.10)" }}>
                <img alt="" src={artURL(art)} width={CHIBI} height={CHIBI} style={{ width: CHIBI, height: CHIBI, objectFit: "contain" }} />
            </div>
        </div>
    );
}

export function DefaultTemplate(data: IDefaultOgData) {
    const { title, subtitle, section, art = "hero" } = data;
    const hero = art === "hero";

    return (
        <CardRoot
            glows={[
                { color: "rgba(236,111,93,0.12)", at: "78% 40%" },
                { color: TEAL_GLOW, at: "10% 95%" },
            ]}
        >
            {hero ? <HeroArt /> : <ChibiArt art={art} />}
            {hero ? <ArtScrim background="linear-gradient(90deg, #0d1216 0%, #0d1216 38%, rgba(13,18,22,0.85) 48%, rgba(13,18,22,0.3) 66%, rgba(13,18,22,0.05) 85%), linear-gradient(0deg, #0d1216 0%, rgba(13,18,22,0.55) 16%, transparent 38%)" /> : null}
            <div style={{ position: "absolute", left: 64, top: 56, width: 1072, display: "flex" }}>
                <BrandRow kicker="ARKNIGHTS · COMPANION" kickerShadow={hero ? "0 1px 8px rgba(0,0,0,0.6)" : undefined} />
            </div>
            <div style={{ position: "absolute", left: 64, top: 112, width: 600, height: 430, display: "flex", flexDirection: "column", justifyContent: "center" }}>
                {section ? (
                    <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
                        <div style={{ display: "flex", width: 28, height: 2, background: "#ec6f5d" }} />
                        <Kicker size={14} tracking="0.24em" color="#f0a093">
                            {section}
                        </Kicker>
                    </div>
                ) : null}
                <div style={{ display: "flex", fontSize: titleFontSize(title), fontWeight: 700, lineHeight: 1, letterSpacing: "-0.028em", color: FG, marginTop: section ? 22 : 0 }}>{title}</div>
                {subtitle ? <div style={{ display: "flex", fontSize: 26, color: FG_55, marginTop: 24, lineHeight: 1.4 }}>{subtitle}</div> : null}
            </div>
            <FootRow />
            <AccentStrip />
        </CardRoot>
    );
}
