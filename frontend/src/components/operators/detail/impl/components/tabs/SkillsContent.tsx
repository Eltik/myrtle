import type { IOperatorListItem } from "#/types/operators";
import { memo } from "react";

interface ISkillsContentProps {
    operator: IOperatorListItem;
}

export const SkillsContent = memo(function SkillsContent({ operator }: ISkillsContentProps) {
    return <></>
});
