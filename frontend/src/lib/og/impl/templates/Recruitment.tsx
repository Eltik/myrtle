import type { IRecruitmentOgData, IRecruitmentOgOperator } from "../recruitment";
import { DefaultTemplate } from "./Default";
import { AccentStrip, BrandRow, CardRoot, Chip, ChipRow, CORAL, FG, FG_06, FG_08, FG_55, FG_70, FootRow, Kicker, RARITY_COLOR, StarIcon, siteHost, TEAL_GLOW } from "./Frame";

/** Name width under a tile: four tiles share the panel's inner width. */
const TILE_W = 96;

/**
 * The headline's outcome: a star rarity, a Robot lock, or no guarantee (a floor
 * of 2 or 3 stars). `accent` paints the glow and strip, coral without a
 * guarantee; `tint` marks the winning tags, the floor's own rarity colour, since
 * coral chip text reads 4.27:1 on the dark fill.
 */
function outcome(floor: number | undefined): { kind: "stars" | "robot" | "none"; accent: string; tint: string } {
    const rarity = (floor != null && RARITY_COLOR[floor]) || CORAL;
    if (floor != null && floor >= 4) return { kind: "stars", accent: rarity, tint: rarity };
    // A Robot lock ranks above a 3 star floor in the calculator (a guaranteed
    // pull), so it is a guarantee here too, not "no guarantee".
    if (floor === 1) return { kind: "robot", accent: rarity, tint: rarity };
    return { kind: "none", accent: CORAL, tint: rarity };
}

function OperatorTile({ op }: { op: IRecruitmentOgOperator }) {
    const color = RARITY_COLOR[op.rarity] ?? FG_55;
    return (
        <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 7, width: TILE_W }}>
            <div style={{ display: "flex", position: "relative", width: 72, height: 72, borderRadius: 10, overflow: "hidden", background: `${color}22`, border: `1px solid ${FG_08}` }}>
                {op.avatarURL ? <img alt="" src={op.avatarURL} width={72} height={72} style={{ width: 72, height: 72, objectFit: "cover" }} /> : null}
                <div style={{ display: "flex", position: "absolute", left: 0, bottom: 0, width: 72, height: 4, background: color }} />
            </div>
            <div style={{ display: "flex", justifyContent: "center", width: TILE_W, fontSize: 13, fontWeight: 700, color: FG_70, lineHeight: 1.1 }}>
                <span style={{ overflow: "hidden", whiteSpace: "nowrap", textOverflow: "ellipsis", maxWidth: TILE_W }}>{op.name}</span>
            </div>
        </div>
    );
}

export function RecruitmentTemplate(data: IRecruitmentOgData) {
    if (data.preset) return DefaultTemplate(data.preset);
    const { tags, best } = data;
    const { kind, accent: color, tint } = outcome(best?.floor);
    const winning = new Set(best?.tagIds ?? []);
    const comboWords = tags.filter((t) => winning.has(t.id)).map((t) => t.name);
    const operators = best?.operators ?? [];
    const more = best ? best.total - operators.length : 0;

    return (
        <CardRoot
            glows={[
                { color: `${color}1c`, at: "82% 40%" },
                { color: TEAL_GLOW, at: "8% 95%" },
            ]}
        >
            <div style={{ position: "absolute", left: 0, top: 0, width: 1200, height: 630, padding: "56px 64px", display: "flex", flexDirection: "column" }}>
                <BrandRow kicker="RECRUITMENT · CALCULATOR" />
                <div style={{ display: "flex", flex: 1, alignItems: "center", gap: 48, marginBottom: 40 }}>
                    <div style={{ display: "flex", flexDirection: "column", justifyContent: "center", width: 560 }}>
                        <Kicker size={14}>Best combination</Kicker>
                        {kind === "stars" ? (
                            <div style={{ display: "flex", alignItems: "center", marginTop: 16, fontSize: 76, fontWeight: 700, lineHeight: 1, letterSpacing: "-0.03em", color: FG }}>
                                <div style={{ display: "flex" }}>{best?.floor}</div>
                                <div style={{ display: "flex", marginLeft: 4, marginRight: 20 }}>
                                    <StarIcon size={60} color={color} />
                                </div>
                                <div style={{ display: "flex" }}>guaranteed</div>
                            </div>
                        ) : (
                            <div style={{ display: "flex", marginTop: 16, fontSize: kind === "robot" ? 62 : 76, fontWeight: 700, lineHeight: 1, letterSpacing: "-0.03em", color: kind === "robot" ? FG : FG_70 }}>{kind === "robot" ? "Robot guaranteed" : "No guarantee"}</div>
                        )}
                        {/* The combination in words only when it is a subset of the selection: for the whole selection the chips already say it. */}
                        {comboWords.length === 0 || comboWords.length < tags.length ? <div style={{ display: "flex", marginTop: 18, fontSize: 26, color: FG_70, lineHeight: 1.25 }}>{comboWords.length > 0 ? comboWords.join(" + ") : "No operator matches these tags"}</div> : null}
                        <ChipRow>
                            {tags.map((t) => (
                                <Chip key={t.id} label={t.name} tint={winning.has(t.id) ? tint : undefined} />
                            ))}
                        </ChipRow>
                    </div>
                    {operators.length > 0 ? (
                        <div style={{ display: "flex", flexDirection: "column", flex: 1, padding: "20px 22px 22px", borderRadius: 12, background: FG_06, border: `1px solid ${FG_08}` }}>
                            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                                <Kicker size={12} color={FG_70}>
                                    Possible operators
                                </Kicker>
                                {more > 0 ? <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: 13, color: FG_55 }}>{`+${more} more`}</div> : null}
                            </div>
                            <div style={{ display: "flex", flexWrap: "wrap", columnGap: 10, rowGap: 16, marginTop: 18 }}>
                                {operators.map((op) => (
                                    <OperatorTile key={op.id} op={op} />
                                ))}
                            </div>
                        </div>
                    ) : null}
                </div>
            </div>
            <FootRow path={`${siteHost()} / tools / recruitment`} />
            <AccentStrip color={color} />
        </CardRoot>
    );
}
