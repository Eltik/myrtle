import { memo } from "react";
import type { IOperatorListItem } from "#/types/operators";

interface IAudioContentProps {
    operator: IOperatorListItem;
}

export const AudioContent = memo(function AudioContent({ operator }: IAudioContentProps) {
    return <></>;
});
