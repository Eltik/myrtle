import { memo } from "react";
import type { IOperatorListItem } from "#/types/operators";

interface IInfoContentProps {
    operator: IOperatorListItem;
}

export const InfoContent = memo(function InfoContent({ operator }: IInfoContentProps) {
    return <></>;
});
