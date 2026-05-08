import { formatProfession } from "#/lib/utils";
import type { OperatorProfession, OperatorRarity } from "#/types/operators";
import { AccentStrip, BG, BrandRow, FG, FG_06, FG_08, FG_55, FG_70, FootRow, RARITY_COLOR, siteHost } from "./Frame";

export interface IOperatorStat {
    label: string;
    value: string;
}

/** Step the name font size down for longer names so the title fits the text column. */
function nameFontSize(name: string): number {
    const len = name.length;
    if (len <= 10) return 112;
    if (len <= 16) return 92;
    if (len <= 22) return 72;
    return 56;
}

export interface IOperatorOgData {
    name: string;
    appellation: string;
    profession: OperatorProfession;
    rarity: OperatorRarity;
    subProfession?: string;
    position?: string;
    nationId?: string;
    factionLabel?: string;
    charArtUrl?: string;
    factionLogoUrl?: string;
    professionIconUrl?: string;
    stats?: IOperatorStat[];
}

export function OperatorTemplate(data: IOperatorOgData) {
    const { name, appellation, profession, rarity, subProfession = "", position = "", nationId = "", factionLabel, charArtUrl, factionLogoUrl, professionIconUrl, stats = [] } = data;
    const rarityColor = RARITY_COLOR[rarity] ?? RARITY_COLOR[6];
    const tags = [position, subProfession, nationId].filter(Boolean);

    // Render the full operator name. Many operators have descriptive suffixes
    // ("Exusiai the New Covenant", "Reed the Flame Shadow") that are part of
    // their canonical name - splitting them off looks worse than letting the
    // size adapt to fit. Appellation is only meaningful when it's a foreign-
    // language alt name; the EN-only data ships it as `" "` (single space),
    // which we filter out with a trim check.
    const altName = appellation.trim();

    return (
        <div
            style={{
                width: 1200,
                height: 630,
                display: "flex",
                background: `radial-gradient(ellipse 70% 60% at 80% 30%, rgba(247,164,82,0.10), transparent 65%), linear-gradient(135deg, #0e141a 0%, #15202a 60%, #0c1218 100%)`,
                color: FG,
                fontFamily: "Inter",
                position: "relative",
                overflow: "hidden",
            }}
        >
            <div
                style={{
                    position: "absolute",
                    right: -40,
                    top: -40,
                    width: 760,
                    height: 760,
                    display: "flex",
                }}
            >
                <div
                    style={{
                        position: "absolute",
                        inset: 0,
                        display: "flex",
                        background: `radial-gradient(circle at 50% 40%, ${rarityColor}33, transparent 55%)`,
                    }}
                />
                {charArtUrl ? <img alt="" src={charArtUrl} width={760} height={760} style={{ width: 760, height: 760, objectFit: "contain", objectPosition: "center top" }} /> : null}
            </div>
            <div
                style={{
                    position: "absolute",
                    inset: 0,
                    display: "flex",
                    background: "linear-gradient(90deg, #0d1216 0%, rgba(13,18,22,0.95) 30%, rgba(13,18,22,0.4) 55%, transparent 75%), linear-gradient(0deg, #0d1216 0%, transparent 30%)",
                }}
            />
            <div
                style={{
                    position: "relative",
                    width: 720,
                    padding: "56px 64px",
                    display: "flex",
                    flexDirection: "column",
                }}
            >
                <BrandRow kicker="OPERATOR · COMPANION" />
                <div style={{ display: "flex", flexDirection: "column", flex: 1, justifyContent: "center", marginTop: 40 }}>
                    <div style={{ display: "flex", alignItems: "center", gap: 14 }}>
                        {/* rarity badge - single inline-SVG star (Inter has no ★ glyph
                            and the rarity_yellow_N.png assets bake N stars in a row) */}
                        <div
                            style={{
                                display: "flex",
                                alignItems: "center",
                                gap: 6,
                                padding: "6px 16px 6px 12px",
                                borderRadius: 999,
                                background: rarityColor,
                                color: "#1c1209",
                                fontWeight: 800,
                                fontSize: 24,
                                letterSpacing: "0.02em",
                                lineHeight: 1,
                            }}
                        >
                            {/* biome-ignore lint/a11y/noSvgWithoutTitle: Satori renders <title> as visible text */}
                            <svg width="22" height="22" viewBox="0 0 24 24" fill="#1c1209">
                                <path d="M12 2.5l2.92 5.92 6.53.95-4.72 4.6 1.11 6.51L12 17.5l-5.84 3.07 1.11-6.51-4.72-4.6 6.53-.95L12 2.5z" />
                            </svg>
                            <div style={{ display: "flex" }}>{rarity}</div>
                        </div>
                        {/* profession as icon + text */}
                        {profession ? (
                            <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                                {professionIconUrl ? (
                                    <div
                                        style={{
                                            display: "flex",
                                            width: 30,
                                            height: 30,
                                            alignItems: "center",
                                            justifyContent: "center",
                                            background: FG_06,
                                            border: `1px solid ${FG_08}`,
                                            borderRadius: 6,
                                        }}
                                    >
                                        <img alt="" src={professionIconUrl} width={22} height={22} style={{ width: 22, height: 22, objectFit: "contain" }} />
                                    </div>
                                ) : null}
                                <div
                                    style={{
                                        display: "flex",
                                        fontFamily: "Geist Mono",
                                        fontSize: 16,
                                        letterSpacing: "0.22em",
                                        textTransform: "uppercase",
                                        color: FG_70,
                                    }}
                                >
                                    {formatProfession(profession)}
                                </div>
                            </div>
                        ) : null}
                    </div>
                    {/* name - full canonical name, font size adapts to length so
                        "Exusiai the New Covenant" still fits on two lines. */}
                    <div
                        style={{
                            display: "flex",
                            fontSize: nameFontSize(name),
                            fontWeight: 700,
                            lineHeight: 0.95,
                            letterSpacing: "-0.025em",
                            color: FG,
                            marginTop: 20,
                            maxWidth: 620,
                        }}
                    >
                        {name}
                    </div>
                    {altName && altName !== name ? (
                        <div
                            style={{
                                display: "flex",
                                fontSize: 28,
                                color: FG_55,
                                marginTop: 16,
                                fontStyle: "italic",
                                fontWeight: 300,
                                maxWidth: 620,
                            }}
                        >
                            {`"${altName}"`}
                        </div>
                    ) : null}
                    {tags.length > 0 ? (
                        <div style={{ display: "flex", gap: 10, flexWrap: "wrap", marginTop: 28 }}>
                            {tags.map((t, i) => (
                                <div
                                    key={t}
                                    style={{
                                        display: "flex",
                                        fontFamily: "Geist Mono",
                                        fontSize: 13,
                                        letterSpacing: "0.12em",
                                        textTransform: "uppercase",
                                        color: i === 0 ? `${rarityColor}d8` : FG_70,
                                        background: i === 0 ? `${rarityColor}10` : FG_06,
                                        border: `1px solid ${i === 0 ? `${rarityColor}55` : FG_08}`,
                                        padding: "7px 12px",
                                        borderRadius: 999,
                                    }}
                                >
                                    {String(t).toUpperCase()}
                                </div>
                            ))}
                        </div>
                    ) : null}
                    {stats.length > 0 ? (
                        <div
                            style={{
                                display: "flex",
                                marginTop: 28,
                                background: FG_06,
                                border: `1px solid ${FG_08}`,
                                borderRadius: 12,
                                overflow: "hidden",
                            }}
                        >
                            {stats.map((s, i) => (
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
                                    <div style={{ display: "flex", fontSize: 22, fontWeight: 700, color: FG, lineHeight: 1, letterSpacing: "-0.01em" }}>{s.value}</div>
                                </div>
                            ))}
                        </div>
                    ) : null}
                </div>
                <div
                    style={{
                        display: "flex",
                        height: 1,
                        background: `linear-gradient(90deg, ${rarityColor}80, transparent 70%)`,
                    }}
                />
            </div>
            {factionLogoUrl ? (
                <div
                    style={{
                        position: "absolute",
                        right: 56,
                        bottom: 64,
                        display: "flex",
                        alignItems: "center",
                        gap: 14,
                        padding: "10px 16px 10px 14px",
                        borderRadius: 12,
                        background: "rgba(13,18,22,0.78)",
                        border: `1px solid ${FG_08}`,
                    }}
                >
                    <div
                        style={{
                            display: "flex",
                            width: 56,
                            height: 56,
                            alignItems: "center",
                            justifyContent: "center",
                            background: BG,
                            border: `1px solid ${FG_08}`,
                            borderRadius: 8,
                        }}
                    >
                        <img alt="" src={factionLogoUrl} width={42} height={42} style={{ width: 42, height: 42, objectFit: "contain" }} />
                    </div>
                    <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
                        <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: 10, letterSpacing: "0.22em", textTransform: "uppercase", color: FG_55 }}>Faction</div>
                        <div style={{ display: "flex", fontSize: 16, fontWeight: 600, color: FG, letterSpacing: "-0.005em", lineHeight: 1 }}>{factionLabel ?? ""}</div>
                    </div>
                </div>
            ) : null}
            <FootRow path={`${siteHost()} / operators`} />
            <AccentStrip color={rarityColor} />
        </div>
    );
}
