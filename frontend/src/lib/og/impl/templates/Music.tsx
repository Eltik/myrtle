import type { IMusicOgData } from "../music";
import { AccentStrip, ArtScrim, BrandRow, CardRoot, Chip, ChipRow, emWidth, FG, FG_08, FG_55, FG_70, FootRow, siteHost, TEAL_GLOW } from "./Frame";

const COVER = 420;
const TEXT_LEFT = 64 + COVER + 56;
const TEXT_W = 1136 - TEXT_LEFT;

function titleFontSize(title: string): number {
    const w = emWidth(title);
    const oneLine = Math.floor(TEXT_W / w);
    if (oneLine >= 52) return Math.min(64, oneLine);
    return Math.max(36, Math.min(52, Math.floor((TEXT_W * 1.9) / w)));
}

export function MusicTemplate(data: IMusicOgData) {
    const { name, artists, trackCount, tracks, coverURL, backdropURL } = data;
    const more = trackCount - tracks.length;

    return (
        <CardRoot
            glows={[
                { color: "rgba(236,111,93,0.08)", at: "80% 30%" },
                { color: TEAL_GLOW, at: "10% 95%" },
            ]}
        >
            {/* The cover's own colours as the background. satori has no blur filter, so the
                handler fetches the cover shrunk to 8 px and it is drawn card-sized: resvg's
                smooth scaling smears it into soft colour fields. */}
            {backdropURL ? (
                <div style={{ position: "absolute", left: 0, top: 0, width: 1200, height: 630, display: "flex", overflow: "hidden" }}>
                    <img alt="" src={backdropURL} width={1400} height={1400} style={{ position: "absolute", left: -100, top: -385, width: 1400, height: 1400, objectFit: "cover", opacity: 0.7 }} />
                </div>
            ) : null}
            <ArtScrim background="linear-gradient(90deg, rgba(13,18,22,0.45) 0%, rgba(13,18,22,0.78) 42%, rgba(13,18,22,0.9) 70%, rgba(13,18,22,0.92) 100%), linear-gradient(0deg, rgba(13,18,22,0.85) 0%, transparent 30%)" />
            <div style={{ position: "absolute", left: 64, top: 56, width: 1072, display: "flex" }}>
                <BrandRow kicker="MUSIC · ALBUM" />
            </div>
            <div style={{ position: "absolute", left: 64, top: 124, width: COVER, height: COVER, display: "flex", borderRadius: 12, overflow: "hidden", border: `1px solid ${FG_08}`, background: "rgba(255,255,255,0.04)" }}>
                {coverURL ? <img alt="" src={coverURL} width={COVER} height={COVER} style={{ width: COVER, height: COVER, objectFit: "cover" }} /> : null}
            </div>
            <div style={{ position: "absolute", left: TEXT_LEFT, top: 124, width: TEXT_W, height: COVER, display: "flex", flexDirection: "column", justifyContent: "center" }}>
                <div style={{ display: "flex", fontSize: titleFontSize(name), fontWeight: 700, lineHeight: 1.02, letterSpacing: "-0.025em", color: FG }}>{name}</div>
                {artists.length > 0 ? <div style={{ display: "flex", marginTop: 14, fontSize: 22, color: FG_70 }}>{artists.join(" · ")}</div> : null}
                <ChipRow marginTop={20}>
                    <Chip label={trackCount === 1 ? "1 track" : `${trackCount} tracks`} />
                </ChipRow>
                {tracks.length > 0 ? (
                    <div style={{ display: "flex", flexDirection: "column", gap: 10, marginTop: 26 }}>
                        {tracks.map((t, i) => (
                            // biome-ignore lint/suspicious/noArrayIndexKey: track order is the identity; two tracks can share a title
                            <div key={i} style={{ display: "flex", alignItems: "baseline", gap: 14 }}>
                                <div style={{ display: "flex", width: 26, fontFamily: "Geist Mono", fontSize: 13, color: FG_55 }}>{String(i + 1).padStart(2, "0")}</div>
                                <div style={{ display: "flex", flex: 1, fontSize: 19, color: FG_70, overflow: "hidden", whiteSpace: "nowrap", textOverflow: "ellipsis" }}>{t}</div>
                            </div>
                        ))}
                        {more > 0 ? <div style={{ display: "flex", marginLeft: 40, fontFamily: "Geist Mono", fontSize: 13, color: FG_55 }}>{`+${more} more`}</div> : null}
                    </div>
                ) : null}
            </div>
            <FootRow path={`${siteHost()} / gallery / music`} />
            <AccentStrip />
        </CardRoot>
    );
}
