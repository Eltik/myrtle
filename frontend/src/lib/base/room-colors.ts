/**
 * Canonical in-game RIIC room accent colours, taken verbatim from
 * `building_data.json` (each buff's `BuffColor`, themed to the room it targets).
 * These are the real base-UI hues - Trading Post blue, Factory gold, Power Plant
 * green, Control Center deep teal - so the plan reads like the game.
 */
const ROOM_COLORS: Record<string, string> = {
    TRADING: "#0075a9", // blue
    MANUFACTURE: "#ffd800", // gold
    POWER: "#8fc31f", // green
    CONTROL: "#005752", // deep teal
    DORMITORY: "#21cdcb", // cyan
    WORKSHOP: "#e3eb00", // yellow
    MEETING: "#dd653f", // orange (Reception)
    HIRE: "#565656", // gray (Office)
    TRAINING: "#7d0022", // maroon
};

/** Raw in-game hue for a room type (falls back to a neutral gray). */
function roomColor(roomType: string): string {
    return ROOM_COLORS[roomType] ?? "#8a8a8a";
}

/**
 * Theme-adaptive shades derived from a room's hue, for the per-room card. Mixing
 * toward `--foreground` keeps even the dark Control Center teal and the bright
 * Factory gold legible on both light and dark backgrounds.
 */
export interface IRoomAccent {
    /** Raw hue (legend dots). */
    color: string;
    /** Coloured shape border around the room's crew. */
    border: string;
    /** Faint fill behind the crew. */
    tint: string;
    /** Room-name label colour. */
    text: string;
    /** Strong accent for figures/badges. */
    strong: string;
}

export function roomAccent(roomType: string): IRoomAccent {
    const c = roomColor(roomType);
    return {
        color: c,
        border: `color-mix(in oklch, ${c} 60%, var(--foreground))`,
        tint: `color-mix(in oklch, ${c} 12%, transparent)`,
        text: `color-mix(in oklch, ${c} 72%, var(--foreground))`,
        strong: `color-mix(in oklch, ${c} 60%, var(--foreground))`,
    };
}
