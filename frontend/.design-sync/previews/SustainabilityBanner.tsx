import type { ReactNode } from "react";
import { SustainabilityBanner } from "frontend";

const op = (operator_id: string, name: string) => ({ operator_id, name });

/** The banner never stands alone — it sits between the shift poster's reading legend
 *  and the shift grid, so the stories reproduce that kicker + banner pairing. */
const PosterSlot = ({ children }: { children: ReactNode }) => (
    <div className="flex max-w-xl flex-col gap-2">
        <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">Rotation sustainability</span>
        {children}
    </div>
);

/** The quiet green line: a week-long morale sim where nobody runs dry. */
export const HoldsUp = () => (
    <PosterSlot>
        <SustainabilityBanner
            sim={{
                verdict: "holds_up",
                horizon_hours: 168,
                depleted: [],
                dorm_overflow: 0,
            }}
        />
    </PosterSlot>
);

/** Operators whose morale empties mid-shift, with when and where. */
export const OperatorsRunDry = () => (
    <PosterSlot>
        <SustainabilityBanner
            sim={{
                verdict: "depletes",
                horizon_hours: 168,
                depleted: [
                    { operator: op("char_190_clour", "Vermeil"), at_hours: 14.2, room_type: "MANUFACTURE" },
                    { operator: op("char_140_whitew", "Lappland"), at_hours: 19.6, room_type: "TRADING" },
                    { operator: op("char_253_greyy", "Greyy"), at_hours: 31.1, room_type: "POWER" },
                ],
                dorm_overflow: 0,
            }}
        />
    </PosterSlot>
);

/** A single depleted operator plus dorms too small to hold everyone resting at peak. */
export const DormOverflow = () => (
    <PosterSlot>
        <SustainabilityBanner
            sim={{
                verdict: "depletes",
                horizon_hours: 168,
                depleted: [{ operator: op("char_164_nightm", "Nightmare"), at_hours: 9.8, room_type: "MANUFACTURE" }],
                dorm_overflow: 2,
            }}
        />
    </PosterSlot>
);

/** Dorm capacity alone is the problem — morale itself holds up. */
export const DormsOnly = () => (
    <PosterSlot>
        <SustainabilityBanner
            sim={{
                verdict: "holds_up",
                horizon_hours: 168,
                depleted: [],
                dorm_overflow: 1,
            }}
        />
    </PosterSlot>
);
