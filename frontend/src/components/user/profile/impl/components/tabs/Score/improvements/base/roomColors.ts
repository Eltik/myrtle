export { type IRoomAccent, roomAccent } from "#/lib/base/room-colors";

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

/**
 * Short formula tag for a factory product ("Gold" / "EXP" / "Shard"), or `null`
 * when the room has no formula configured. Unknown `F_*` codes fall back to a
 * title-cased form of the code instead of vanishing.
 */
export function formulaTag(formula: string | null | undefined): string | null {
    if (!formula) return null;
    if (formula === "F_GOLD") return "Gold";
    if (formula === "F_EXP") return "EXP";
    if (formula === "F_DIAMOND") return "Shard";
    const bare = formula.replace(/^F_/, "");
    if (!bare) return null;
    return bare.charAt(0) + bare.slice(1).toLowerCase();
}

/** Room name with its formula tag appended ("Factory · Gold"). */
export function roomFormulaLabel(roomType: string, formula: string | null | undefined): string {
    const tag = formulaTag(formula);
    return tag ? `${roomLabel(roomType)} · ${tag}` : roomLabel(roomType);
}
