import type { IStage, IZone } from "#/types/stages";
import { AccentStrip, BrandRow, FG, FG_06, FG_08, FG_45, FG_55, FG_70, FootRow, siteHost } from "./Frame";

const ACCENT = "#5aa9d9";

const STAGE_TYPE_LABEL: Record<string, string> = {
    MAIN: "Main Theme",
    SUB: "Sub Stage",
    ACTIVITY: "Event",
    DAILY: "Daily / Resource",
    CAMPAIGN: "Campaign",
    CLIMB_TOWER: "Stationary Security",
    GUIDE: "Tutorial",
    SPECIAL_STORY: "Special Story",
};

const DIFFICULTY_LABEL: Record<string, string> = {
    FOUR_STAR: "Challenge Mode",
    SIX_STAR: "Extreme",
};

/** Reduce a stage's rich-text description to a plain, single-line OG blurb. */
function stripStageText(text: string): string {
    return text
        .replace(/<[@$][a-z0-9._]+>(.*?)<\/>/gi, "$1")
        .replace(/<\/?[^>]+>/g, "")
        .replace(/\\n|\n/g, " ")
        .replace(/\s+/g, " ")
        .trim();
}

function truncate(text: string, max: number): string {
    if (text.length <= max) return text;
    return `${text.slice(0, max - 1).trimEnd()}…`;
}

function zoneDisplay(zone: IZone | undefined, fallback: string): string {
    if (!zone) return fallback;
    return zone.zoneNameSecond || zone.zoneNameFirst || zone.zoneNameTitleCurrent || zone.zoneId;
}

/** Step the title font size down for longer stage names so it fits the column. */
function nameFontSize(name: string): number {
    const len = name.length;
    if (len <= 16) return 76;
    if (len <= 26) return 60;
    if (len <= 38) return 48;
    return 40;
}

export interface IStageOgStat {
    label: string;
    value: string;
}

export interface IStageOgData {
    code: string;
    name: string;
    description?: string;
    zoneName: string;
    typeLabel: string;
    difficultyLabel?: string;
    bossMark?: boolean;
    stats: IStageOgStat[];
    previewImageURL?: string;
}

/** Shared shape used by both the route (for hashing) and the OG handler (for render). */
export function buildStageOgData(stage: IStage, zone: IZone | undefined, previewImageURL?: string): IStageOgData {
    return {
        code: stage.code,
        name: stage.name ?? stage.code,
        description: stage.description ? truncate(stripStageText(stage.description), 180) : undefined,
        zoneName: zoneDisplay(zone, stage.zoneId),
        typeLabel: STAGE_TYPE_LABEL[stage.stageType] ?? stage.stageType,
        difficultyLabel: stage.difficulty !== "NORMAL" ? (DIFFICULTY_LABEL[stage.difficulty] ?? stage.difficulty) : undefined,
        bossMark: stage.bossMark,
        stats: [
            { label: "Sanity", value: String(stage.apCost) },
            { label: "EXP", value: stage.expGain.toLocaleString("en-US") },
            { label: "LMD", value: stage.goldGain.toLocaleString("en-US") },
        ],
        previewImageURL,
    };
}

export function StageTemplate(data: IStageOgData) {
    const { code, name, description, zoneName, typeLabel, difficultyLabel, bossMark, stats, previewImageURL } = data;

    return (
        <div
            style={{
                width: 1200,
                height: 630,
                display: "flex",
                background: `radial-gradient(ellipse 70% 60% at 82% 35%, rgba(90,169,217,0.12), transparent 65%), linear-gradient(135deg, #0e141a 0%, #15202a 60%, #0c1218 100%)`,
                color: FG,
                fontFamily: "Inter",
                position: "relative",
                overflow: "hidden",
            }}
        >
            <div
                style={{
                    position: "absolute",
                    right: 48,
                    top: 96,
                    width: 520,
                    height: 392,
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    borderRadius: 16,
                    border: `1px solid ${FG_08}`,
                    background: "rgba(8,12,16,0.55)",
                    overflow: "hidden",
                }}
            >
                <div
                    style={{
                        position: "absolute",
                        inset: 0,
                        display: "flex",
                        background: `radial-gradient(circle at 50% 40%, ${ACCENT}22, transparent 60%)`,
                    }}
                />
                {previewImageURL ? <img alt="" src={previewImageURL} width={520} height={392} style={{ width: 520, height: 392, objectFit: "contain" }} /> : <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: 14, letterSpacing: "0.2em", textTransform: "uppercase", color: FG_45 }}>No Map Preview</div>}
            </div>
            <div
                style={{
                    position: "absolute",
                    inset: 0,
                    display: "flex",
                    background: "linear-gradient(90deg, #0d1216 0%, rgba(13,18,22,0.96) 38%, rgba(13,18,22,0.55) 56%, transparent 70%)",
                }}
            />
            <div
                style={{
                    position: "relative",
                    width: 760,
                    padding: "56px 64px",
                    display: "flex",
                    flexDirection: "column",
                }}
            >
                <BrandRow kicker="STAGE · COMPANION" />
                <div style={{ display: "flex", flexDirection: "column", flex: 1, justifyContent: "center", marginTop: 28 }}>
                    <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
                        <div
                            style={{
                                display: "flex",
                                padding: "8px 16px",
                                borderRadius: 8,
                                background: ACCENT,
                                color: "#06131c",
                                fontFamily: "Geist Mono",
                                fontWeight: 800,
                                fontSize: 26,
                                letterSpacing: "0.02em",
                                lineHeight: 1,
                            }}
                        >
                            {code}
                        </div>
                        {bossMark ? (
                            <div
                                style={{
                                    display: "flex",
                                    fontFamily: "Geist Mono",
                                    fontSize: 13,
                                    letterSpacing: "0.14em",
                                    textTransform: "uppercase",
                                    color: "#f0a3a3",
                                    background: "rgba(220,77,86,0.12)",
                                    border: "1px solid rgba(220,77,86,0.45)",
                                    padding: "7px 12px",
                                    borderRadius: 999,
                                }}
                            >
                                Boss
                            </div>
                        ) : null}
                        {difficultyLabel ? (
                            <div
                                style={{
                                    display: "flex",
                                    fontFamily: "Geist Mono",
                                    fontSize: 13,
                                    letterSpacing: "0.14em",
                                    textTransform: "uppercase",
                                    color: "#e6c779",
                                    background: "rgba(216,181,74,0.12)",
                                    border: "1px solid rgba(216,181,74,0.45)",
                                    padding: "7px 12px",
                                    borderRadius: 999,
                                }}
                            >
                                {difficultyLabel}
                            </div>
                        ) : null}
                    </div>
                    <div
                        style={{
                            display: "flex",
                            fontSize: nameFontSize(name),
                            fontWeight: 700,
                            lineHeight: 0.98,
                            letterSpacing: "-0.025em",
                            color: FG,
                            marginTop: 22,
                            maxWidth: 640,
                        }}
                    >
                        {name}
                    </div>
                    <div
                        style={{
                            display: "flex",
                            gap: 10,
                            alignItems: "center",
                            fontFamily: "Geist Mono",
                            fontSize: 14,
                            letterSpacing: "0.14em",
                            textTransform: "uppercase",
                            color: FG_70,
                            marginTop: 18,
                        }}
                    >
                        <div style={{ display: "flex" }}>{typeLabel}</div>
                        <div style={{ display: "flex", color: FG_45 }}>·</div>
                        <div style={{ display: "flex", color: FG }}>{zoneName}</div>
                    </div>
                    {description ? (
                        <div
                            style={{
                                display: "flex",
                                fontSize: 20,
                                lineHeight: 1.4,
                                color: FG_55,
                                marginTop: 22,
                                maxWidth: 600,
                            }}
                        >
                            {description}
                        </div>
                    ) : null}
                    <div
                        style={{
                            display: "flex",
                            marginTop: 30,
                            background: FG_06,
                            border: `1px solid ${FG_08}`,
                            borderRadius: 12,
                            overflow: "hidden",
                            maxWidth: 600,
                        }}
                    >
                        {stats.map((s, i) => (
                            <div
                                key={s.label}
                                style={{
                                    flex: 1,
                                    display: "flex",
                                    flexDirection: "column",
                                    gap: 6,
                                    padding: "14px 18px",
                                    borderLeft: i === 0 ? "none" : `1px solid ${FG_08}`,
                                }}
                            >
                                <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: 10, letterSpacing: "0.22em", textTransform: "uppercase", color: FG_55 }}>{s.label}</div>
                                <div style={{ display: "flex", fontSize: 24, fontWeight: 700, color: FG, lineHeight: 1, letterSpacing: "-0.01em" }}>{s.value}</div>
                            </div>
                        ))}
                    </div>
                </div>
                <div
                    style={{
                        display: "flex",
                        height: 1,
                        background: `linear-gradient(90deg, ${ACCENT}80, transparent 70%)`,
                    }}
                />
            </div>
            <FootRow path={`${siteHost()} / stages`} />
            <AccentStrip color={ACCENT} />
        </div>
    );
}
