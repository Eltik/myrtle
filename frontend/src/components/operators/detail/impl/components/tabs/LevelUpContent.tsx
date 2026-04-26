import type { IOperatorListItem } from "#/types/operators";
import { memo } from "react";

interface ILevelUpContentProps {
    operator: IOperatorListItem;
}

export const LevelUpContent = memo(function LevelUpContent({ operator }: ILevelUpContentProps) {
    return <></>
});
