import type { IOperatorListItem } from "#/types/operators";
import { calculateBirthdays } from "./impl/calculate";

interface IBirthdaysProps {
    operators: IOperatorListItem[];
}

export function Birthdays({ operators }: IBirthdaysProps) {
    const bdays = calculateBirthdays(operators);
    console.log(bdays);
    return null;
}
