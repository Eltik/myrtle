import { LevelPips } from "frontend";
import type { CSSProperties, ReactNode } from "react";

/*
 * LevelPips lives in a room tile's name row: `max` is the room's phase count
 * from the catalog (3 for output rooms, 5 for dorms and the Control Center)
 * and `current` the built level. A lit pip takes the tile's `--tile-accent`
 * when the tile sets one (trading / factory / power), else plain white.
 */

const ACCENT = { TRADING: "#38c9fe", MANUFACTURE: "#fec904", POWER: "#c3ff43" } as const;

/** A strip of the tile's slab so the unlit pips (white at 20%) read. */
const Slab = ({ accent, children }: { accent?: string; children: ReactNode }) => (
    <div className="flex flex-col gap-3" style={{ "--tile-accent": accent, width: "fit-content", minWidth: 200, background: "#323232", border: "1px solid rgb(255 255 255 / 0.08)", padding: "12px 14px", color: "#fff" } as CSSProperties}>
        {children}
    </div>
);

const Row = ({ label, children }: { label: string; children: ReactNode }) => (
    <div className="flex items-center justify-between gap-6">
        <span style={{ fontSize: 11, fontWeight: 700, letterSpacing: 0.1 }}>{label}</span>
        {children}
    </div>
);

/** Every fill of a 3-phase and a 5-phase room, as they sit beside the room name. */
export const Sweep = () => (
    <div className="flex flex-wrap items-start gap-4">
        <Slab accent={ACCENT.MANUFACTURE}>
            {[1, 2, 3].map((level) => (
                <Row key={level} label={`Factory · Lv ${level}`}>
                    <LevelPips current={level} max={3} />
                </Row>
            ))}
        </Slab>
        <Slab>
            {[1, 2, 3, 4, 5].map((level) => (
                <Row key={level} label={`Dormitory · Lv ${level}`}>
                    <LevelPips current={level} max={5} />
                </Row>
            ))}
        </Slab>
    </div>
);

/** The lit colour follows the tile's accent: cyan for trading, yellow for factories, lime for power, white where no accent is set. */
export const Accents = () => (
    <div className="flex flex-wrap items-start gap-4">
        <Slab accent={ACCENT.TRADING}>
            <Row label="Trading Post · Lv 3">
                <LevelPips current={3} max={3} />
            </Row>
        </Slab>
        <Slab accent={ACCENT.MANUFACTURE}>
            <Row label="Factory · Lv 2">
                <LevelPips current={2} max={3} />
            </Row>
        </Slab>
        <Slab accent={ACCENT.POWER}>
            <Row label="Power Plant · Lv 3">
                <LevelPips current={3} max={3} />
            </Row>
        </Slab>
        <Slab>
            <Row label="Reception Room · Lv 3">
                <LevelPips current={3} max={3} />
            </Row>
        </Slab>
    </div>
);
