import { professionLabel } from "#/lib/utils";
import type { OperatorProfession, OperatorRarity } from "#/types/operators";
import { AccentStrip, BG, BrandRow, CardRoot, Chip, ChipRow, FG, FG_06, FG_08, FG_55, FG_70, FootRow, Kicker, RARITY_COLOR, StarPill, StatStrip, siteHost } from "./Frame";

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
    /** The server's own class name; the English formatter names the class without it. */
    professionName?: string;
    rarity: OperatorRarity;
    subProfession?: string;
    position?: string;
    nationId?: string;
    factionLabel?: string;
    charArtURL?: string;
    factionLogoURL?: string;
    professionIconURL?: string;
    stats?: IOperatorStat[];
    /** Asset tree the card was rendered against. Not drawn: it is in the cache
     *  hash so a CN operator's card re-renders once it lands on Global and its
     *  art moves to the default tree. */
    server?: "en" | "cn";
}

export function OperatorTemplate(data: IOperatorOgData) {
    const { name, appellation, profession, professionName, rarity, subProfession = "", position = "", nationId = "", factionLabel, charArtURL, factionLogoURL, professionIconURL, stats = [] } = data;
    const rarityColor = RARITY_COLOR[rarity] ?? RARITY_COLOR[6];
    const tags = [position, subProfession, nationId].filter(Boolean);

    // Suffixes like "Exusiai the New Covenant" are part of the canonical name;
    // splitting them off looked worse than shrinking the size to fit. EN data
    // ships an absent appellation as `" "`, hence the trim.
    const altName = appellation.trim();

    return (
        <CardRoot>
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
                {charArtURL ? <img alt="" src={charArtURL} width={760} height={760} style={{ width: 760, height: 760, objectFit: "contain", objectPosition: "center top" }} /> : null}
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
                        <StarPill label={rarity} color={rarityColor} />
                        {profession ? (
                            <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                                {professionIconURL ? (
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
                                        <img alt="" src={professionIconURL} width={22} height={22} style={{ width: 22, height: 22, objectFit: "contain" }} />
                                    </div>
                                ) : null}
                                <Kicker size={16} color={FG_70}>
                                    {professionLabel({ profession, professionName })}
                                </Kicker>
                            </div>
                        ) : null}
                    </div>
                    {/* The full canonical name; the font size adapts to length so
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
                        <ChipRow>
                            {tags.map((t, i) => (
                                <Chip key={t} label={String(t)} tint={i === 0 ? rarityColor : undefined} />
                            ))}
                        </ChipRow>
                    ) : null}
                    {stats.length > 0 ? <StatStrip cells={stats} /> : null}
                </div>
                <div
                    style={{
                        display: "flex",
                        height: 1,
                        background: `linear-gradient(90deg, ${rarityColor}80, transparent 70%)`,
                    }}
                />
            </div>
            {factionLogoURL ? (
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
                        <img alt="" src={factionLogoURL} width={42} height={42} style={{ width: 42, height: 42, objectFit: "contain" }} />
                    </div>
                    <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
                        <Kicker size={10}>Faction</Kicker>
                        <div style={{ display: "flex", fontSize: 16, fontWeight: 600, color: FG, letterSpacing: "-0.005em", lineHeight: 1 }}>{factionLabel ?? ""}</div>
                    </div>
                </div>
            ) : null}
            <FootRow path={`${siteHost()} / operators`} />
            <AccentStrip color={rarityColor} />
        </CardRoot>
    );
}
