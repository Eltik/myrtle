import { memo } from "react";
import type { IOperatorListItem } from "#/types/operators";

interface ILoreContentProps {
    operator: IOperatorListItem;
}

export const LoreContent = memo(function LoreContent({ operator }: ILoreContentProps) {
    return <></>;
});
