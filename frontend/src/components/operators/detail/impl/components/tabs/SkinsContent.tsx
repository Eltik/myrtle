import { memo } from "react";
import type { IOperatorListItem } from "#/types/operators";

interface ISkinsContentProps {
    operator: IOperatorListItem;
}

export const SkinsContent = memo(function SkinsContent({ operator }: ISkinsContentProps) {
    return <></>;
});
