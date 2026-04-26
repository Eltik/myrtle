import type { IOperatorListItem } from "#/types/operators";
import { memo } from "react";

interface ILoreContentProps {
    operator: IOperatorListItem;
}

export const LoreContent = memo(function LoreContent({ operator }: ILoreContentProps) {
    return <></>
});
