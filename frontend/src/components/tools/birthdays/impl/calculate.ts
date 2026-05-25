import type { IOperatorListItem } from "#/types/operators";
import type { IOperatorBirthday } from "./types";

const BIRTHDAY_RE = /^([a-z]+)\.?\s+(\d{1,2})$/i;

const MONTHS: Record<string, number> = {
    jan: 1,
    feb: 2,
    mar: 3,
    apr: 4,
    may: 5,
    jun: 6,
    jul: 7,
    aug: 8,
    sep: 9,
    oct: 10,
    nov: 11,
    dec: 12,
};

function parseBirthday(raw: string): { month: number; day: number } | null {
    const m = BIRTHDAY_RE.exec(raw);
    if (!m) return null;
    const month = MONTHS[m[1].slice(0, 3).toLowerCase()];
    const day = Number(m[2]);
    if (!month || day < 1 || day > 31) return null;
    return { month, day };
}

export function calculateBirthdays(operators: IOperatorListItem[]): IOperatorBirthday[] {
    return operators.map((operator) => {
        const raw = operator.profile?.basicInfo.dateOfBirth?.trim() ?? "";
        const parsed = raw ? parseBirthday(raw) : null;
        return parsed ? { operator, known: true, raw, ...parsed } : { operator, known: false, raw };
    });
}
