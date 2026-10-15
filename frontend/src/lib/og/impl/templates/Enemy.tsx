import { formatNumber } from "#/lib/utils";
import type { DamageType } from "#/types/generated/DamageType";
import type { EnemyLevel } from "#/types/generated/EnemyLevel";
import type { IEnemyOgData } from "../enemy";
import { AccentStrip, BrandRow, CardRoot, Chip, ChipRow, CORAL, emWidth, FG, FG_06, FG_08, FG_70, FootRow, StatStrip, siteHost, TEAL_GLOW } from "./Frame";

/** The level's colour: ELITE is oklch(0.72 0.14 75) as hex, BOSS the brand coral, NORMAL a quiet slate. */
const LEVEL_COLOR: Record<EnemyLevel, string> = {
    NORMAL: "#9aa7b4",
    ELITE: "#d79628",
    BOSS: CORAL,
};

const DAMAGE: Record<DamageType, { label: string; color?: string }> = {
    PHYSIC: { label: "Physical", color: "#d39e6a" },
    MAGIC: { label: "Arts", color: "#b78fe6" },
    HEAL: { label: "Healing", color: "#7ed3a2" },
    NO_DAMAGE: { label: "No damage" },
};

const MOTION: Record<string, string> = { WALK: "Ground", FLY: "Aerial" };

/** The icon's drawn size: the file is 158 px, so this is about 1.65x, the most before its edges soften visibly. */
const ICON = 260;

/** The name's size in the 600 px column: one line where it fits at 60 or more, else two lines. */
function nameFontSize(name: string): number {
    const w = emWidth(name);
    const oneLine = Math.floor(600 / w);
    if (oneLine >= 60) return Math.min(92, oneLine);
    return Math.max(44, Math.min(64, Math.floor(1120 / w)));
}

/** Clamp to about two lines of the 21 px description in the 600 px column (about 55 characters a line): satori has no line clamp. */
function clampDescription(text: string, max = 108): string {
    if (text.length <= max) return text;
    const cut = text.slice(0, max);
    return `${cut.slice(0, Math.max(cut.lastIndexOf(" "), max - 20)).replace(/[,.;:\s]+$/, "")}...`;
}

function LevelBadge({ level }: { level: EnemyLevel }) {
    const color = LEVEL_COLOR[level];
    if (level === "NORMAL") return <Chip label="Normal" />;
    return <div style={{ display: "flex", alignItems: "center", padding: "7px 14px", borderRadius: 999, background: color, color: "#1c1209", fontFamily: "Geist Mono", fontSize: 13, fontWeight: 700, letterSpacing: "0.16em" }}>{level === "BOSS" ? "BOSS" : "ELITE"}</div>;
}

export function EnemyTemplate(data: IEnemyOgData) {
    const { name, index, level, description, motion, damageTypes, stats, iconURL } = data;
    const color = LEVEL_COLOR[level];
    const cells = stats
        ? [
              { label: "HP", value: formatNumber(stats.hp) },
              { label: "ATK", value: formatNumber(stats.atk) },
              { label: "DEF", value: formatNumber(stats.def) },
              { label: "RES", value: formatNumber(stats.res) },
          ]
        : [];
    const motionLabel = motion ? MOTION[motion] : undefined;

    return (
        <CardRoot
            glows={[
                { color: `${color}24`, at: "78% 45%" },
                { color: TEAL_GLOW, at: "8% 95%" },
            ]}
        >
            <div style={{ position: "absolute", left: 0, top: 0, width: 1200, height: 630, padding: "56px 64px", display: "flex", flexDirection: "column" }}>
                <BrandRow kicker="ENEMY · DATABASE" />
                <div style={{ display: "flex", flex: 1, alignItems: "center", gap: 56, marginTop: 8, marginBottom: 28 }}>
                    <div style={{ display: "flex", flexDirection: "column", width: 620 }}>
                        <div style={{ display: "flex", alignItems: "center", gap: 14 }}>
                            <LevelBadge level={level} />
                            {index ? <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: 18, fontWeight: 700, letterSpacing: "0.12em", color: FG_70 }}>{index}</div> : null}
                        </div>
                        <div style={{ display: "flex", marginTop: 18, maxWidth: 620, fontSize: nameFontSize(name), fontWeight: 700, lineHeight: 1, letterSpacing: "-0.028em", color: FG }}>{name}</div>
                        {motionLabel || damageTypes.length > 0 ? (
                            <ChipRow marginTop={22}>
                                {motionLabel ? <Chip label={motionLabel} /> : null}
                                {damageTypes.map((d) => (
                                    <Chip key={d} label={DAMAGE[d]?.label ?? d} tint={DAMAGE[d]?.color} />
                                ))}
                            </ChipRow>
                        ) : null}
                        {description ? <div style={{ display: "flex", marginTop: 20, maxWidth: 600, fontSize: 21, lineHeight: 1.4, color: FG_70 }}>{clampDescription(description)}</div> : null}
                        {cells.length > 0 ? <StatStrip cells={cells} marginTop={24} /> : null}
                    </div>
                    <div
                        style={{
                            display: "flex",
                            position: "relative",
                            width: 396,
                            height: 396,
                            alignItems: "center",
                            justifyContent: "center",
                            borderRadius: 16,
                            overflow: "hidden",
                            border: `1px solid ${FG_08}`,
                            background: `radial-gradient(circle at 50% 50%, ${color}66, ${color}1f 50%, transparent 75%), repeating-linear-gradient(135deg, ${FG_06} 0px, ${FG_06} 2px, transparent 2px, transparent 12px), rgba(13,18,22,0.6)`,
                        }}
                    >
                        {iconURL ? <img alt="" src={iconURL} width={ICON} height={ICON} style={{ width: ICON, height: ICON, objectFit: "contain", borderRadius: 10 }} /> : null}
                        <div style={{ display: "flex", position: "absolute", left: 0, top: 0, width: 396, height: 3, background: color }} />
                    </div>
                </div>
            </div>
            <FootRow path={`${siteHost()} / enemies`} />
            <AccentStrip color={color} />
        </CardRoot>
    );
}
