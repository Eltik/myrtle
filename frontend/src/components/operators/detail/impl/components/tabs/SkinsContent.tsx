import type { IOperatorListItem } from "#/types/operators";
import { memo } from "react";

interface ISkinsContentProps {
    operator: IOperatorListItem;
}

export const SkinsContent = memo(function SkinsContent({ operator }: ISkinsContentProps) {
    return <></>
});
