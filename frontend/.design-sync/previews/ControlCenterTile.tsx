import { ControlCenterTile } from "frontend";
import type { CSSProperties, ReactNode } from "react";

/*
 * The Control Center is the base's one fixed 8x4 slot (`slot_34` in
 * `GET /api/base/catalog`): four tracks wide, two tall. Its tile carries a
 * version badge instead of the name row + level pips a room gets. Seats per
 * level are 1..5.
 */
interface ICrew {
    id: string;
    name: string;
    skills: never[];
    change?: "added" | "removed";
}

const SEATS = [1, 2, 3, 4, 5];

const op = (id: string, name: string): ICrew => ({ id, name, skills: [] });

const AMIYA = op("char_002_amiya", "Amiya");
const KALTSIT = op("char_003_kalts", "Kal'tsit");
const SILVERASH = op("char_172_svrash", "SilverAsh");
const SWIRE = op("char_308_swire", "Swire");
const CHEN = op("char_010_chen", "Ch'en");

/** An ITile for the Control Center at `level`, placed at grid column `col`. */
function controlCenter(level: number, operators: ICrew[], col = 1) {
    return {
        slotId: "slot_34",
        kind: "fixed" as const,
        facility: "CONTROL" as const,
        name: "Control Center",
        level,
        maxPhase: SEATS.length,
        built: true,
        operators,
        seats: SEATS[level - 1],
        col,
        row: 1,
        w: 4,
        h: 2,
    };
}

/** The board's stage: `Board.module.css`'s dark surface and half-tile grid, at the desktop unit. */
const Stage = ({ tracks, children }: { tracks: number; children: ReactNode }) => (
    <div style={{ width: "fit-content", borderRadius: 6, background: "#191919", padding: "22px 16px" }}>
        <div style={{ "--riic-unit": "32px", display: "grid", gap: 3, width: "max-content", gridTemplateColumns: Array.from({ length: tracks }, () => "calc(2 * var(--riic-unit))").join(" "), gridAutoRows: "calc(2 * var(--riic-unit))" } as CSSProperties}>{children}</div>
    </div>
);

/** A maxed Control Center with a full five-seat crew: the version badge reads the level as "Ver 5.0". */
export const Level5 = () => (
    <Stage tracks={4}>
        <ControlCenterTile tile={controlCenter(5, [AMIYA, KALTSIT, SILVERASH, SWIRE, CHEN])} />
    </Stage>
);

/** Level 1, 3 and 5 side by side - the badge's number is the level and the seat strip grows with it (the level-3 one has a seat open). */
export const LevelSweep = () => (
    <div className="flex flex-col gap-2">
        <Stage tracks={12}>
            <ControlCenterTile tile={controlCenter(1, [AMIYA], 1)} />
            <ControlCenterTile tile={controlCenter(3, [AMIYA, KALTSIT], 5)} />
            <ControlCenterTile tile={controlCenter(5, [AMIYA, KALTSIT, SILVERASH, SWIRE, CHEN], 9)} />
        </Stage>
        <p className="font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.1em]">Ver 1.0 · Ver 3.0 (2 of 3 seated) · Ver 5.0</p>
    </div>
);
