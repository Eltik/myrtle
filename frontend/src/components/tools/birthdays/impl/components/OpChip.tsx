import type * as React from "react";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { cn } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";
import { operatorRarity, rarityVar } from "../helpers";

type OpChipSize = "sm" | "default" | "lg" | "xl";

const SIZE_CLASS: Record<OpChipSize, string> = {
    sm: "size-6 rounded-md text-[11px]",
    default: "size-9 rounded-lg text-[14px]",
    lg: "size-12 rounded-xl text-[17px]",
    xl: "size-16 rounded-2xl text-[22px]",
};

interface IOpChipProps {
    operator: IOperatorListItem;
    size?: OpChipSize;
    className?: string;
}

/** Rarity-tinted operator tile: shows the portrait once loaded, else the operator's initial. */
export function OpChip({ operator, size = "default", className }: IOpChipProps): React.ReactElement {
    return (
        <span
            className={cn("relative inline-flex shrink-0 items-center justify-center overflow-hidden font-bold font-sans text-white leading-none tracking-[-0.01em] [box-shadow:inset_0_1px_0_rgba(255,255,255,0.25),inset_0_-1px_0_rgba(0,0,0,0.2)]", SIZE_CLASS[size], className)}
            style={{ backgroundColor: rarityVar(operatorRarity(operator)) }}
            title={operator.name}
        >
            <OperatorAvatar charId={operator.id} name={operator.name} />
        </span>
    );
}
