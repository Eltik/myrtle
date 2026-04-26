import type { IOperatorListItem } from "#/types/operators";
import { memo } from "react";

interface IAudioContentProps {
    operator: IOperatorListItem;
}

export const AudioContent = memo(function AudioContent({ operator }: IAudioContentProps) {
    return <></>
});
