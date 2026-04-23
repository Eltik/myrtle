import type { IOperatorListItem } from "#/types/operators";

interface IOperatorCardCompactProps {
    operator: IOperatorListItem;
}

export function OperatorCardCompact({ operator }: IOperatorCardCompactProps) {
    const initial = operator.name.charAt(0).toUpperCase();
    return <></>;
}
