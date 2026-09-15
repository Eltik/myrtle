import type { OperatorProfession } from "#/types/operators";
import type { messages as birthdayConstantsMessages } from "./constants.messages";
import type { CalendarScale } from "./types";

/** A key in `constants.messages.ts`; resolved by whichever component renders it. */
export type BirthdayMessageKey = keyof typeof birthdayConstantsMessages & string;

/** Rarity chips, highest first. */
export const RARITIES = [6, 5, 4, 3, 2, 1] as const;

/** Calendar zoom levels, narrowest first. */
export const CALENDAR_SCALES: { id: CalendarScale; labelKey: BirthdayMessageKey }[] = [
    { id: "day", labelKey: "birthdays.scale.day" },
    { id: "3day", labelKey: "birthdays.scale.3day" },
    { id: "week", labelKey: "birthdays.scale.week" },
    { id: "month", labelKey: "birthdays.scale.month" },
];

/** The eight playable classes, in the game's canonical order. Labels come from `formatProfession`. */
export const PROFESSIONS: OperatorProfession[] = ["PIONEER", "WARRIOR", "TANK", "SNIPER", "CASTER", "MEDIC", "SUPPORT", "SPECIAL"];

/** Professions excluded from the calendar - summons and map hazards aren't operators. */
export const NON_OPERATOR_PROFESSIONS = new Set<string>(["TOKEN", "TRAP"]);
