import { memo } from "react";
import type { IOperatorListItem } from "#/types/operators";

interface ILevelUpContentProps {
    operator: IOperatorListItem;
}

export const LevelUpContent = memo(function LevelUpContent({ operator }: ILevelUpContentProps) {
    return <></>;
});
