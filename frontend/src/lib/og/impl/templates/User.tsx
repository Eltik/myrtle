import { BG, BrandMark, FG, FG_06, FG_08, FG_45, FG_55, FG_70, FootRow, RARITY_COLOR, RainbowStrip } from "./Frame";

export interface IUserSupportSkill {
    iconUrl?: string;
    mastery: number;
    skillLevel: number;
    /** /textures/arts/specialized_hub/specialized_{mastery}.png — only set when mastery >= 1. */
    masteryIconUrl?: string;
}

export interface IUserSupportModule {
    iconUrl?: string;
    level: number;
}

export interface IUserSupportUnit {
    id: string;
    name: string;
    rarity: number;
    elite: number;
    level: number;
    avatarUrl?: string;
    skills?: IUserSupportSkill[];
    modules?: IUserSupportModule[];
}

export interface IUserOgData {
    nickname: string;
    uid?: string;
    level: number | null;
    grade: string | null;
    totalScore: number | null;
    operatorCount?: number;
    skinCount?: number;
    itemCount?: number;
    lmd?: number;
    secretaryArtUrl?: string;
    supportUnits?: IUserSupportUnit[];
    supportUnitsKind?: "supports" | "roster";
    rarityCounts?: Record<number, number>;
}

const fmt = (n: number | null | undefined) => Number(n ?? 0).toLocaleString("en-US");
const compact = (n: number | null | undefined) => {
    const v = Number(n ?? 0);
    if (v >= 1_000_000) return `${(v / 1_000_000).toFixed(1).replace(/\.0$/, "")}M`;
    if (v >= 1_000) return `${(v / 1_000).toFixed(1).replace(/\.0$/, "")}k`;
    return fmt(v);
};

export function UserTemplate(data: IUserOgData) {
    const { nickname, uid = "—", level = 1, grade = "B", operatorCount = 0, skinCount = 0, itemCount = 0, lmd = 0, totalScore = 0, secretaryArtUrl, supportUnits, supportUnitsKind = "roster", rarityCounts } = data;

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
            <div
                style={{
                    width: 660,
                    height: 630,
                    display: "flex",
                    position: "relative",
                    background: "linear-gradient(135deg, #2b3a4a 0%, #11171c 70%)",
                }}
            >
                {secretaryArtUrl ? <img alt="" src={secretaryArtUrl} width={720} height={700} style={{ position: "absolute", left: -30, top: -20, width: 720, height: 700, objectFit: "cover" }} /> : null}
                <div
                    style={{
                        position: "absolute",
                        top: 0,
                        left: 0,
                        width: 660,
                        height: 630,
                        display: "flex",
                        background: "linear-gradient(180deg, transparent 0%, rgba(13,18,22,0.55) 100%)",
                    }}
                />
                <div
                    style={{
                        position: "absolute",
                        left: 0,
                        bottom: 0,
                        width: 660,
                        height: 280,
                        display: "flex",
                        background: "linear-gradient(180deg, transparent 0%, #0d1216 85%)",
                    }}
                />
                <div
                    style={{
                        position: "absolute",
                        left: 0,
                        bottom: 0,
                        width: 520,
                        height: 360,
                        display: "flex",
                        background: "linear-gradient(45deg, #0d1216 0%, transparent 60%)",
                    }}
                />
                <div
                    style={{
                        position: "absolute",
                        top: 0,
                        bottom: 0,
                        right: 0,
                        width: 70,
                        display: "flex",
                        background: "linear-gradient(90deg, transparent, #0d1216 80%)",
                    }}
                />
                <div
                    style={{
                        position: "absolute",
                        left: 56,
                        bottom: 56,
                        display: "flex",
                        flexDirection: "column",
                        maxWidth: 540,
                    }}
                >
                    <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: 13, letterSpacing: "0.2em", textTransform: "uppercase", color: FG_55 }}>DOCTOR</div>
                    <div
                        style={{
                            display: "flex",
                            fontSize: 84,
                            fontWeight: 700,
                            lineHeight: 0.95,
                            letterSpacing: "-0.025em",
                            color: FG,
                            marginTop: 8,
                        }}
                    >
                        {nickname}
                    </div>
                    <div
                        style={{
                            display: "flex",
                            alignItems: "center",
                            gap: 12,
                            fontFamily: "Geist Mono",
                            fontSize: 16,
                            letterSpacing: "0.08em",
                            color: FG_70,
                            marginTop: 14,
                        }}
                    >
                        <div style={{ display: "flex" }}>Lv {level ?? "—"}</div>
                        <div style={{ display: "flex", width: 4, height: 4, borderRadius: 999, background: FG_45 }} />
                        <div
                            style={{
                                display: "flex",
                                padding: "3px 11px",
                                borderRadius: 999,
                                background: "linear-gradient(135deg, #ec6f5d, #c63a48)",
                                color: "#1a0c0c",
                                fontFamily: "Inter",
                                fontWeight: 700,
                                fontSize: 14,
                                letterSpacing: "0.05em",
                            }}
                        >
                            {grade ?? "—"}
                        </div>
                        <div style={{ display: "flex", width: 4, height: 4, borderRadius: 999, background: FG_45 }} />
                        <div style={{ display: "flex" }}>{fmt(totalScore)} pts</div>
                    </div>
                </div>
            </div>
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
                        UID&nbsp;·&nbsp;<span style={{ color: FG_70, marginLeft: 4 }}>{uid}</span>
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
                        { k: "Operators", v: fmt(operatorCount), red: true },
                        { k: "Skins", v: fmt(skinCount), red: false },
                        { k: "Items", v: compact(itemCount), red: false },
                        { k: "LMD", v: compact(lmd), red: false },
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
                {supportUnits && supportUnits.length > 0 ? <SupportUnits units={supportUnits} kind={supportUnitsKind} /> : null}
            </div>
            <FootRow />
            <RarityDistributionBar counts={rarityCounts} />
        </div>
    );
}

function SupportUnits({ units, kind }: { units: IUserSupportUnit[]; kind: "supports" | "roster" }) {
    const top = units.slice(0, 3);
    const label = kind === "supports" ? "Support Units" : "Top Operators";
    return (
        <div
            style={{
                display: "flex",
                flexDirection: "column",
                gap: 8,
            }}
        >
            <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
                <div style={{ display: "flex", height: 1, width: 16, background: FG_45 }} />
                <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: 10, letterSpacing: "0.24em", textTransform: "uppercase", color: FG_70 }}>{label}</div>
            </div>
            <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
                {top.map((u) => (
                    <SupportUnitRow key={u.id} unit={u} />
                ))}
            </div>
        </div>
    );
}

function SupportUnitRow({ unit }: { unit: IUserSupportUnit }) {
    const color = RARITY_COLOR[unit.rarity] ?? FG;
    const skills = (unit.skills ?? []).slice(0, 3);
    const modules = (unit.modules ?? []).slice(0, 3);

    return (
        <div
            style={{
                display: "flex",
                gap: 10,
                background: FG_06,
                border: `1px solid ${FG_08}`,
                borderRadius: 10,
                borderLeft: `3px solid ${color}`,
                padding: "8px 10px",
                alignItems: "center",
            }}
        >
            <div
                style={{
                    display: "flex",
                    width: 52,
                    height: 52,
                    borderRadius: 8,
                    background: `linear-gradient(135deg, ${color}33, transparent 70%)`,
                    border: `1px solid ${color}55`,
                    overflow: "hidden",
                    alignItems: "center",
                    justifyContent: "center",
                    flexShrink: 0,
                }}
            >
                {unit.avatarUrl ? <img alt="" src={unit.avatarUrl} width={52} height={52} style={{ width: 52, height: 52, objectFit: "cover" }} /> : null}
            </div>
            <div style={{ display: "flex", flexDirection: "column", gap: 4, width: 110, flexShrink: 0 }}>
                <div style={{ display: "flex", fontSize: 13, fontWeight: 700, color: FG, lineHeight: 1.05, letterSpacing: "-0.01em" }}>{unit.name}</div>
                <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                    <div
                        style={{
                            display: "flex",
                            padding: "1px 5px",
                            borderRadius: 4,
                            background: `${color}22`,
                            color,
                            fontFamily: "Geist Mono",
                            fontWeight: 700,
                            fontSize: 9,
                            letterSpacing: "0.06em",
                            lineHeight: 1.4,
                        }}
                    >
                        E{unit.elite}
                    </div>
                    <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: 10, letterSpacing: "0.08em", color: FG_70 }}>LV {unit.level}</div>
                </div>
            </div>
            <div style={{ display: "flex", gap: 4, flexShrink: 0 }}>
                {skills.map((s, i) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: skill index is a stable positional slot
                    <IconCell key={i} iconUrl={s.iconUrl} badge={s.mastery >= 1 ? `M${s.mastery}` : `Lv${s.skillLevel}`} badgeIconUrl={s.mastery >= 1 ? s.masteryIconUrl : undefined} highlight={s.mastery >= 1} />
                ))}
            </div>
            {modules.length > 0 ? <div style={{ display: "flex", width: 1, height: 36, background: FG_08, flexShrink: 0 }} /> : null}
            {modules.length > 0 ? (
                <div style={{ display: "flex", gap: 4, flexShrink: 0 }}>
                    {modules.map((m, i) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: module slot is a stable positional index
                        <IconCell key={i} iconUrl={m.iconUrl} badge={m.level > 0 ? `L${m.level}` : "MOD"} dim={m.level === 0} />
                    ))}
                </div>
            ) : null}
        </div>
    );
}

function RarityDistributionBar({ counts }: { counts?: Record<number, number> }) {
    const total = counts ? [1, 2, 3, 4, 5, 6].reduce((sum, r) => sum + (counts[r] ?? 0), 0) : 0;
    if (!counts || total === 0) return <RainbowStrip />;

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
            }}
        >
            {[6, 5, 4, 3, 2, 1].map((rarity) => {
                const count = counts[rarity] ?? 0;
                if (count === 0) return null;
                const pct = (count / total) * 100;
                return <div key={rarity} style={{ display: "flex", width: `${pct}%`, height: 6, background: RARITY_COLOR[rarity] }} />;
            })}
        </div>
    );
}

function IconCell({ iconUrl, badge, badgeIconUrl, highlight = false, dim = false }: { iconUrl?: string; badge: string; badgeIconUrl?: string; highlight?: boolean; dim?: boolean }) {
    return (
        <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 2, opacity: dim ? 0.45 : 1 }}>
            <div
                style={{
                    display: "flex",
                    width: 32,
                    height: 32,
                    borderRadius: 5,
                    background: "rgba(255,255,255,0.04)",
                    border: `1px solid ${FG_08}`,
                    alignItems: "center",
                    justifyContent: "center",
                    overflow: "hidden",
                }}
            >
                {iconUrl ? <img alt="" src={iconUrl} width={32} height={32} style={{ width: 32, height: 32, objectFit: "contain" }} /> : null}
            </div>
            {badgeIconUrl ? (
                <img alt="" src={badgeIconUrl} width={16} height={16} style={{ width: 16, height: 16, objectFit: "contain" }} />
            ) : (
                <div
                    style={{
                        display: "flex",
                        fontFamily: "Geist Mono",
                        fontSize: 8,
                        letterSpacing: "0.04em",
                        color: highlight ? "#ffce95" : FG_55,
                        fontWeight: highlight ? 700 : 500,
                        lineHeight: 1,
                    }}
                >
                    {badge}
                </div>
            )}
        </div>
    );
}
