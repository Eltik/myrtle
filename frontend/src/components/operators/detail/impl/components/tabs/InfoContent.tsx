import type { IOperatorListItem } from "#/types/operators";
import { memo } from "react";

interface IInfoContentProps {
    operator: IOperatorListItem;
}

export const InfoContent = memo(function InfoContent({ operator }: IInfoContentProps) {
    return <></>;
});
