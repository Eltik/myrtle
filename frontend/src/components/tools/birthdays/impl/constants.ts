import type { OperatorProfession } from "#/types/operators";
import type { CalendarScale } from "./types";

export const MONTHS = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"] as const;
export const MONTHS_SHORT = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"] as const;
export const WEEKDAYS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"] as const;

/** Rarity chips, highest first. */
export const RARITIES = [6, 5, 4, 3, 2, 1] as const;

/** Calendar zoom levels, narrowest first. */
export const CALENDAR_SCALES: { id: CalendarScale; label: string }[] = [
    { id: "day", label: "Day" },
    { id: "3day", label: "3 Day" },
    { id: "week", label: "Week" },
    { id: "month", label: "Month" },
];

/** The eight playable classes, in the game's canonical order. Labels come from `formatProfession`. */
export const PROFESSIONS: OperatorProfession[] = ["PIONEER", "WARRIOR", "TANK", "SNIPER", "CASTER", "MEDIC", "SUPPORT", "SPECIAL"];

/** Professions excluded from the calendar - summons and map hazards aren't operators. */
export const NON_OPERATOR_PROFESSIONS = new Set<string>(["TOKEN", "TRAP"]);
