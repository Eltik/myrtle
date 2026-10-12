import type { ISquadOgData, ISquadOgSeat } from "../squad";
import { DefaultTemplate } from "./Default";
import { AccentStrip, BrandRow, CardRoot, Chip, ChipRow, CORAL, emWidth, FG, FG_08, FG_45, FG_55, FG_70, FootRow, Kicker, RARITY_COLOR, siteHost, TEAL_GLOW } from "./Frame";

/** One seat's avatar edge. Six to a row, two rows, beside the support column. */
const TILE = 104;
const GAP = 16;

/** The title's size in the 760 px header: one line, stepping down with width. */
function titleFontSize(title: string): number {
    return Math.max(40, Math.min(64, Math.floor(760 / emWidth(title))));
}

/** `E2 · S3 · M3`: the elite phase and mastery only when the link carries a build, the skill always. */
function seatBadge(seat: ISquadOgSeat): string {
    const parts = [];
    if (seat.elite !== null) parts.push(`E${seat.elite}`);
    parts.push(`S${seat.skill + 1}`);
    if (seat.mastery) parts.push(`M${seat.mastery}`);
    return parts.join(" · ");
}

function SeatTile({ seat, support = false }: { seat: ISquadOgSeat | null; support?: boolean }) {
    if (!seat) {
        return (
            <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 7, width: TILE }}>
                <div style={{ display: "flex", width: TILE, height: TILE, borderRadius: 10, border: "1px dashed rgba(255,255,255,0.14)", background: "rgba(255,255,255,0.02)" }} />
                <div style={{ display: "flex", height: 13 }} />
            </div>
        );
    }
    const color = (seat.rarity && RARITY_COLOR[seat.rarity]) || FG_45;
    return (
        <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 7, width: TILE }}>
            <div
                style={{
                    display: "flex",
                    position: "relative",
                    width: TILE,
                    height: TILE,
                    borderRadius: 10,
                    overflow: "hidden",
                    background: `${color}22`,
                    border: support ? `2px solid ${CORAL}` : `1px solid ${FG_08}`,
                }}
            >
                {seat.avatarURL ? <img alt="" src={seat.avatarURL} width={TILE} height={TILE} style={{ width: TILE, height: TILE, objectFit: "cover" }} /> : null}
                <div style={{ display: "flex", position: "absolute", left: 0, bottom: 0, width: TILE, height: 4, background: color }} />
            </div>
            <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: 12, fontWeight: 700, letterSpacing: "0.06em", color: FG_70, lineHeight: 1 }}>{seatBadge(seat)}</div>
        </div>
    );
}

export function SquadTemplate(data: ISquadOgData) {
    if (data.preset) return DefaultTemplate(data.preset);
    const { name, seats, support, stage } = data;
    const filled = seats.filter((s) => s !== null).length;
    const title = name || "Shared squad";
    const rows = [seats.slice(0, 6), seats.slice(6, 12)];

    return (
        <CardRoot
            glows={[
                { color: "rgba(236,111,93,0.10)", at: "80% 30%" },
                { color: TEAL_GLOW, at: "8% 95%" },
            ]}
        >
            <div style={{ position: "absolute", left: 0, top: 0, width: 1200, height: 630, padding: "56px 64px", display: "flex", flexDirection: "column" }}>
                <BrandRow kicker="SQUAD · BUILDER" />
                <div style={{ display: "flex", flexDirection: "column", flex: 1, justifyContent: "center", marginBottom: 36 }}>
                    <div style={{ display: "flex", fontSize: titleFontSize(title), fontWeight: 700, lineHeight: 1, letterSpacing: "-0.028em", color: name ? FG : FG_70 }}>{title}</div>
                    <ChipRow marginTop={18}>
                        <Chip label={`${filled} / 12 operators`} />
                        {stage ? <Chip label={stage.name ? `${stage.code} ${stage.name}` : stage.code} tint={CORAL} /> : null}
                    </ChipRow>
                    <div style={{ display: "flex", alignItems: "flex-end", gap: 40, marginTop: 30 }}>
                        <div style={{ display: "flex", flexDirection: "column", gap: 14 }}>
                            {rows.map((row, r) => (
                                // biome-ignore lint/suspicious/noArrayIndexKey: the two rows are fixed positions
                                <div key={r} style={{ display: "flex", gap: GAP }}>
                                    {row.map((seat, i) => (
                                        // biome-ignore lint/suspicious/noArrayIndexKey: a seat's index is its identity; empty seats have no id
                                        <SeatTile key={i} seat={seat} />
                                    ))}
                                </div>
                            ))}
                        </div>
                        <div style={{ display: "flex", width: 1, height: 230, background: FG_08 }} />
                        <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 12 }}>
                            <Kicker size={12} color={support ? "#f0a093" : FG_55}>
                                Support
                            </Kicker>
                            <SeatTile seat={support} support />
                        </div>
                    </div>
                </div>
            </div>
            <FootRow path={`${siteHost()} / tools / squad-builder`} />
            <AccentStrip color={CORAL} />
        </CardRoot>
    );
}
