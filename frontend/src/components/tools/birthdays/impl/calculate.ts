import type { IBirthdayOperator, IOperatorBirthday } from "./types";

/** EN "Jan 7", "Mar. 7", "December 25": a month name, read by its first three letters, then the day. */
const BIRTHDAY_RE = /^([a-z]+)\.?\s+(\d{1,2})$/i;
/** KR "1월 7일", JP and CN "1月7日": numeric month and day with the CJK unit markers. */
const BIRTHDAY_CJK_RE = /^(\d{1,2})\s*[월月]\s*(\d{1,2})\s*[일日]?$/;

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

export function parseBirthday(raw: string): { month: number; day: number } | null {
    const cjk = BIRTHDAY_CJK_RE.exec(raw);
    const m = cjk ?? BIRTHDAY_RE.exec(raw);
    if (!m) return null;
    const month = cjk ? Number(cjk[1]) : MONTHS[m[1].slice(0, 3).toLowerCase()];
    const day = Number(m[2]);
    if (!month || month > 12 || day < 1 || day > 31) return null;
    return { month, day };
}

export function calculateBirthdays(operators: IBirthdayOperator[]): IOperatorBirthday[] {
    return operators.map((operator) => {
        // An index cached by a backend older than this field has no dateOfBirth.
        const raw = (operator.dateOfBirth ?? "").trim();
        const parsed = raw ? parseBirthday(raw) : null;
        return parsed ? { operator, known: true, raw, ...parsed } : { operator, known: false, raw };
    });
}
