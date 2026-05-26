import type { IOperatorListItem, OperatorProfession } from "#/types/operators";

export type IOperatorBirthday = { operator: IOperatorListItem; known: false; raw: string } | { operator: IOperatorListItem; known: true; raw: string; month: number; day: number };

export type BirthdayView = "calendar" | "list" | "upcoming";

/** Zoom level for the calendar view. */
export type CalendarScale = "day" | "3day" | "week" | "month";

export interface IBirthdayFilters {
    query: string;
    /** Numeric rarities (1-6) the operator must match one of, when non-empty. */
    rarities: Set<number>;
    /** Profession ids the operator must match one of, when non-empty. */
    professions: Set<OperatorProfession>;
    /** `nationId`s the operator must match one of, when non-empty. */
    nations: Set<string>;
}

/** A focused calendar day. `month` is 1-12. */
export interface ISelectedDay {
    year: number;
    month: number;
    day: number;
}
