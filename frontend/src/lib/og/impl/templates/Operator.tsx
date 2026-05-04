import type { OperatorProfession, OperatorRarity } from "#/types/operators";
import { BrandRow, FG, FG_06, FG_08, FG_45, FG_55, FG_70, FootRow, RARITY_COLOR, RainbowStrip, siteHost } from "./Frame";

export interface IOperatorOgData {
    name: string;
    appellation: string;
    profession: OperatorProfession;
    rarity: OperatorRarity;
    subProfession?: string;
    position?: string;
    nationId?: string;
    charArtUrl?: string;
    factionLogoUrl?: string;
}

export function OperatorTemplate(data: IOperatorOgData) {
    const { name, appellation, profession, rarity, subProfession = "", position = "", nationId = "", charArtUrl, factionLogoUrl } = data;
    const rarityColor = RARITY_COLOR[rarity] ?? RARITY_COLOR[6];
    const tags = [position, subProfession, nationId].filter(Boolean);

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
                        <div
                            style={{
                                display: "flex",
                                alignItems: "center",
                                gap: 6,
                                padding: "6px 16px",
                                borderRadius: 999,
                                background: rarityColor,
                                color: "#1c1209",
                                fontWeight: 800,
                                fontSize: 22,
                                letterSpacing: "0.02em",
                            }}
                        >
                            <div style={{ display: "flex", fontSize: 24, lineHeight: 1 }}>★</div>
                            <div style={{ display: "flex" }}>{rarity}</div>
                        </div>
                        {profession ? (
                            <div
                                style={{
                                    display: "flex",
                                    fontFamily: "Geist Mono",
                                    fontSize: 18,
                                    letterSpacing: "0.22em",
                                    textTransform: "uppercase",
                                    color: FG_70,
                                }}
                            >
                                {profession}
                            </div>
                        ) : null}
                    </div>
                    <div
                        style={{
                            display: "flex",
                            fontSize: name.length > 14 ? 88 : 108,
                            fontWeight: 700,
                            lineHeight: 0.95,
                            letterSpacing: "-0.025em",
                            color: FG,
                            marginTop: 20,
                            maxWidth: 700,
                        }}
                    >
                        {name}
                    </div>
                    {appellation ? (
                        <div
                            style={{
                                display: "flex",
                                fontSize: 28,
                                color: FG_55,
                                marginTop: 16,
                                fontStyle: "italic",
                                fontWeight: 300,
                            }}
                        >
                            {`"${appellation}"`}
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
                        right: 64,
                        bottom: 56,
                        display: "flex",
                        alignItems: "center",
                        gap: 12,
                        fontFamily: "Geist Mono",
                        fontSize: 12,
                        letterSpacing: "0.18em",
                        textTransform: "uppercase",
                        color: FG_45,
                    }}
                >
                    <div style={{ display: "flex" }}>FACTION</div>
                    <img alt="" src={factionLogoUrl} width={56} height={56} style={{ width: 56, height: 56, opacity: 0.85 }} />
                </div>
            ) : null}
            <FootRow path={`${siteHost()} / operators`} />
            <RainbowStrip />
        </div>
    );
}
