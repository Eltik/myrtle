import { LayoutChips } from "frontend";

const LAYOUT = [
    { room_type: "TRADING", count: 2, levels: [3, 3] },
    { room_type: "MANUFACTURE", count: 4, levels: [3, 3, 2, 2] },
    { room_type: "POWER", count: 3, levels: [3, 3, 2] },
    { room_type: "DORMITORY", count: 4, levels: [5, 4, 3, 1] },
    { room_type: "CONTROL", count: 1, levels: [5] },
    { room_type: "MEETING", count: 1, levels: [3] },
    { room_type: "HIRE", count: 1, levels: [3] },
    { room_type: "WORKSHOP", count: 1, levels: [3] },
    { room_type: "TRAINING", count: 1, levels: [3] },
];

/** The collapsed panel view: every room with its count and per-room levels. */
export const CurrentLayoutDetailed = () => (
    <div className="flex max-w-2xl flex-col gap-2">
        <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">Current layout</span>
        <LayoutChips layout={LAYOUT} detailed />
    </div>
);

/** The compact form headers the full-plan dialog, doubling as the room-colour legend
 *  for the shift grid below it. */
export const PlanHeaderLegend = () => (
    <div className="flex max-w-xl flex-col gap-2.5 border-border/40 border-b pb-3">
        <LayoutChips layout={LAYOUT} />
        <span className="text-[10.5px] text-muted-foreground leading-snug">Room colours below match this legend.</span>
    </div>
);

/** An early-account base: three rooms, all level 1–2. */
export const EarlyAccount = () => (
    <div className="flex max-w-md flex-col gap-2">
        <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">Current layout</span>
        <LayoutChips
            layout={[
                { room_type: "TRADING", count: 1, levels: [2] },
                { room_type: "MANUFACTURE", count: 2, levels: [2, 1] },
                { room_type: "POWER", count: 1, levels: [1] },
                { room_type: "DORMITORY", count: 1, levels: [2] },
            ]}
            detailed
        />
    </div>
);
