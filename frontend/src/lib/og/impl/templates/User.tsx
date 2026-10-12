import { formatNumber, formatNumberCompact } from "#/lib/utils";
import type { IUserOgData, IUserOgTile } from "../user";
import { AccentStrip, ArtScrim, BrandRow, CardRoot, Chip, ChipRow, CORAL, emWidth, FG, FG_06, FG_08, FG_55, FG_70, FootRow, Kicker, RARITY_COLOR, StatStrip, siteHost, TEAL_GLOW, WARM_GLOW } from "./Frame";

/** The hero art's box: the right ~65% of the card, full height. */
const ART_LEFT = 420;
const ART_W = 1200 - ART_LEFT;
const ART_H = 630;

/** The nickname column's width beside the 96 px avatar. */
const NAME_MAX_W = 460;

/**
 * The grade colours of the profile's Score tab (`GRADE_COLORS`, oklch there),
 * as hex: satori reads no oklch, and a tinted chip appends alpha digits. B, C,
 * D and F are lifted from the page's values, which read 3.3:1 to 4.5:1 as chip
 * text on the card's dark fill; these clear 4.5:1 as chip text and as a stat.
 */
const GRADE_HEX: Record<string, string> = {
    "S+": "#f9a300",
    S: "#e89800",
    A: "#54b85b",
    B: "#1cc3cc",
    C: "#5aa6ff",
    D: "#b08aff",
    F: "#ff7a72",
};

/** The nickname's size: the largest step that keeps it on one line in the column, down to a floor that may wrap. */
function nicknameFontSize(nickname: string): number {
    const w = emWidth(nickname);
    for (const size of [80, 68, 58, 50, 44]) {
        if (w * size <= NAME_MAX_W) return size;
    }
    return 38;
}

/** The hero art, `cover`-fit to the art box at the background's focus and zoom. Zoom is drawn as a larger cover box placed so the focus point stays put, which is what the page's `transform: scale` about the focus does. */
function HeroArt({ url, position, zoom }: { url: string; position: string; zoom: number }) {
    const s = zoom / 100;
    const [px, py] = position.split(" ").map((p) => Number.parseFloat(p) / 100);
    const w = Math.round(ART_W * s);
    const h = Math.round(ART_H * s);
    const left = Math.round((px ?? 0.5) * (ART_W - w));
    const top = Math.round((py ?? 0.5) * (ART_H - h));
    return (
        <div style={{ position: "absolute", left: ART_LEFT, top: 0, width: ART_W, height: ART_H, display: "flex", overflow: "hidden" }}>
            <img alt="" src={url} width={w} height={h} style={{ position: "absolute", left, top, width: w, height: h, objectFit: "cover", objectPosition: position }} />
        </div>
    );
}

function Avatar({ url }: { url?: string }) {
    return <div style={{ display: "flex", width: 96, height: 96, flexShrink: 0, borderRadius: 12, overflow: "hidden", background: FG_06, border: `1px solid ${FG_08}` }}>{url ? <img alt="" src={url} width={94} height={94} style={{ width: 94, height: 94, objectFit: "cover" }} /> : null}</div>;
}

/** A compact unit tile: the avatar on a rarity bar, elite and level, the drawn skill with its mastery. */
function UnitTile({ tile }: { tile: IUserOgTile }) {
    const color = RARITY_COLOR[tile.rarity] ?? FG_55;
    return (
        <div style={{ display: "flex", alignItems: "center", gap: 10, padding: 8, borderRadius: 12, background: "rgba(13,18,22,0.82)", border: `1px solid ${FG_08}` }}>
            <div style={{ display: "flex", position: "relative", width: 64, height: 64, borderRadius: 8, overflow: "hidden", background: `${color}22` }}>
                {tile.avatarURL ? <img alt="" src={tile.avatarURL} width={64} height={64} style={{ width: 64, height: 64, objectFit: "cover" }} /> : null}
                <div style={{ display: "flex", position: "absolute", left: 0, bottom: 0, width: 64, height: 3, background: color }} />
            </div>
            <div style={{ display: "flex", flexDirection: "column", gap: 5, width: 46 }}>
                <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: 11, fontWeight: 700, letterSpacing: "0.08em", color: FG_70, lineHeight: 1 }}>{`E${tile.elite}`}</div>
                <div style={{ display: "flex", fontSize: 16, fontWeight: 700, color: FG, lineHeight: 1, letterSpacing: "-0.01em" }}>{`Lv ${tile.level}`}</div>
                <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
                    <div style={{ display: "flex", width: 22, height: 22, borderRadius: 4, overflow: "hidden", background: FG_06, border: `1px solid ${FG_08}` }}>{tile.skillIconURL ? <img alt="" src={tile.skillIconURL} width={20} height={20} style={{ width: 20, height: 20, objectFit: "contain" }} /> : null}</div>
                    {tile.mastery > 0 ? <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: 10, fontWeight: 700, color: "#ffce95", lineHeight: 1 }}>{`M${tile.mastery}`}</div> : null}
                </div>
            </div>
        </div>
    );
}

/** Bottom accent split by the roster's rarity counts, 6 stars first; the plain coral strip without a roster. */
function RarityDistributionBar({ counts }: { counts?: Record<number, number> }) {
    const total = counts ? [1, 2, 3, 4, 5, 6].reduce((sum, r) => sum + (counts[r] ?? 0), 0) : 0;
    if (!counts || total === 0) return <AccentStrip />;
    return (
        <div style={{ position: "absolute", left: 0, bottom: 0, width: 1200, height: 6, display: "flex" }}>
            {[6, 5, 4, 3, 2, 1].map((rarity) => {
                const count = counts[rarity] ?? 0;
                if (count === 0) return null;
                return <div key={rarity} style={{ display: "flex", width: `${(count / total) * 100}%`, height: 6, background: RARITY_COLOR[rarity] }} />;
            })}
        </div>
    );
}

export function UserTemplate(data: IUserOgData) {
    const { nickname, nickNumber, server, level, grade, totalScore, operatorCount, skinCount, extraStat, art } = data;
    const tiles = (art?.tiles ?? []).slice(0, 3);
    const gradeColor = grade ? (GRADE_HEX[grade] ?? CORAL) : undefined;

    // Score leads while the Score tab is public; a private one drops the cell
    // rather than drawing a zero.
    const cells = [
        ...(totalScore != null ? [{ label: "Score", value: `${(totalScore * 100).toFixed(1)}%`, color: gradeColor }] : []),
        { label: "Operators", value: operatorCount != null ? formatNumber(operatorCount) : "-" },
        { label: "Skins", value: skinCount != null ? formatNumber(skinCount) : "-" },
        extraStat.kind === "lmd" ? { label: "LMD", value: extraStat.value != null ? formatNumberCompact(extraStat.value) : "-" } : { label: "Items", value: extraStat.value != null ? formatNumber(extraStat.value) : "-" },
    ];

    return (
        <CardRoot glows={[{ color: WARM_GLOW }, { color: TEAL_GLOW, at: "10% 95%" }]}>
            {art?.heroURL ? <HeroArt url={art.heroURL} position={art.heroPosition} zoom={art.heroZoom} /> : null}
            <ArtScrim background="linear-gradient(90deg, #0d1216 0%, #0d1216 36%, rgba(13,18,22,0.82) 46%, rgba(13,18,22,0.25) 64%, transparent 80%), linear-gradient(0deg, #0d1216 0%, rgba(13,18,22,0.7) 18%, transparent 42%)" />
            {/* The header spans the card, so the kicker sits at the right edge over the art, shadowed to stay legible on a bright one. */}
            <div style={{ position: "absolute", left: 64, top: 56, width: 1072, display: "flex" }}>
                <BrandRow kicker="PLAYER · PROFILE" kickerShadow="0 1px 8px rgba(0,0,0,0.6)" />
            </div>
            <div style={{ position: "absolute", left: 0, top: 0, width: 720, height: 630, padding: "56px 64px", display: "flex", flexDirection: "column" }}>
                <div style={{ display: "flex", height: 28 }} />
                <div style={{ display: "flex", flexDirection: "column", flex: 1, justifyContent: "center", marginTop: 8, marginBottom: 40 }}>
                    <div style={{ display: "flex", alignItems: "center", gap: 24 }}>
                        <Avatar url={art?.avatarURL} />
                        <div style={{ display: "flex", flexDirection: "column", maxWidth: NAME_MAX_W }}>
                            <div style={{ display: "flex", fontSize: nicknameFontSize(nickname), fontWeight: 700, lineHeight: 1, letterSpacing: "-0.025em", color: FG }}>{nickname}</div>
                            {nickNumber ? <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: 22, fontWeight: 500, letterSpacing: "0.04em", color: FG_55, marginTop: 10, lineHeight: 1 }}>{`#${nickNumber}`}</div> : null}
                        </div>
                    </div>
                    <ChipRow>
                        <Chip label={server} />
                        {level != null ? <Chip label={`LV ${level}`} /> : null}
                        {grade && gradeColor ? <Chip label={`Grade ${grade}`} tint={gradeColor} /> : null}
                    </ChipRow>
                    <StatStrip cells={cells} />
                </div>
            </div>
            {tiles.length > 0 ? (
                <div style={{ position: "absolute", right: 56, bottom: 72, display: "flex", flexDirection: "column", alignItems: "flex-end", gap: 10 }}>
                    <Kicker size={11} color={FG_70}>
                        {art?.tilesKind === "supports" ? "Support Units" : "Top Operators"}
                    </Kicker>
                    <div style={{ display: "flex", gap: 8 }}>
                        {tiles.map((t, i) => (
                            // biome-ignore lint/suspicious/noArrayIndexKey: a tile's slot is its identity; one operator can fill two slots
                            <UnitTile key={i} tile={t} />
                        ))}
                    </div>
                </div>
            ) : null}
            <FootRow path={`${siteHost()} / user`} />
            <RarityDistributionBar counts={art?.rarityCounts} />
        </CardRoot>
    );
}
