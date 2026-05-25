import type { IOperatorListItem } from "#/types/operators";

export type IOperatorBirthday = { operator: IOperatorListItem; known: false; raw: string } | { operator: IOperatorListItem; known: true; raw: string; month: number; day: number };
