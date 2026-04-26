import { memo } from "react";
import type { IOperatorListItem } from "#/types/operators";

interface ISkillsContentProps {
    operator: IOperatorListItem;
}

export const SkillsContent = memo(function SkillsContent({ operator }: ISkillsContentProps) {
    return <></>;
});
