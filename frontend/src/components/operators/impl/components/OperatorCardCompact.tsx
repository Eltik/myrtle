import { cn, formatSubProfession } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";
import { Link } from "@tanstack/react-router";
import { ClassIcon } from "./ClassIcon";

interface IOperatorCardCompactProps {
    operator: IOperatorListItem;
}

export function OperatorCardCompact({ operator }: IOperatorCardCompactProps) {
    const initial = operator.name.charAt(0).toUpperCase();
    return <></>;
}
