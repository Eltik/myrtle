/**
 * Canonical in-game RIIC room accent colours, taken verbatim from
 * `building_data.json` (each buff's `BuffColor`, themed to the room it targets).
 * These are the real base-UI hues - Trading Post blue, Factory gold, Power Plant
 * green, Control Center deep teal - so the plan reads like the game.
 */
export const ROOM_COLORS: Record<string, string> = {
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
export function roomColor(roomType: string): string {
    return ROOM_COLORS[roomType] ?? "#8a8a8a";
}

/**
 * Theme-adaptive shades derived from a room's hue, for the per-room card. Mixing
 * toward `--foreground` keeps even the dark Control Center teal and the bright
 * Factory gold legible on both light and dark backgrounds.
 */
export interface RoomAccent {
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

export function roomAccent(roomType: string): RoomAccent {
    const c = roomColor(roomType);
    return {
        color: c,
        border: `color-mix(in oklch, ${c} 60%, var(--foreground))`,
        tint: `color-mix(in oklch, ${c} 12%, transparent)`,
        text: `color-mix(in oklch, ${c} 72%, var(--foreground))`,
        strong: `color-mix(in oklch, ${c} 60%, var(--foreground))`,
    };
}

const ROOM_LABELS: Record<string, string> = {
    MANUFACTURE: "Factory",
    TRADING: "Trading Post",
    POWER: "Power Plant",
    DORMITORY: "Dormitory",
    WORKSHOP: "Workshop",
    MEETING: "Reception",
    HIRE: "Office",
    TRAINING: "Training",
    CONTROL: "Control Center",
};

/** Display name for a room type. */
export function roomLabel(roomType: string): string {
    return ROOM_LABELS[roomType] ?? roomType.charAt(0) + roomType.slice(1).toLowerCase();
}

/** Short formula tag for a factory (Gold / EXP / Shard), or `null`. */
export function formulaTag(formula: string | null | undefined): string | null {
    if (formula === "F_GOLD") return "Gold";
    if (formula === "F_EXP") return "EXP";
    if (formula === "F_DIAMOND") return "Shard";
    return null;
}

/** Room name with its formula tag appended ("Factory · Gold"). */
export function roomFormulaLabel(roomType: string, formula: string | null | undefined): string {
    const tag = formulaTag(formula);
    return tag ? `${roomLabel(roomType)} · ${tag}` : roomLabel(roomType);
}

/** Distinct room types present in a plan, in a stable display order, for the legend. */
export const ROOM_LEGEND_ORDER = ["TRADING", "MANUFACTURE", "POWER", "CONTROL", "DORMITORY"] as const;
