import { Flag, RotateCcw, Sparkles, Target } from "lucide-react";
import type React from "react";
import { Button } from "#/components/ui/button";
import type { ChallengeKind, IChallenge } from "../types";
import { SlabFrame } from "./SlabFrame";

interface IModifierSlabProps {
    challenge: IChallenge;
    onReroll: () => void;
}

const KIND_ICON: Record<ChallengeKind, typeof Flag> = {
    restriction: Flag,
    modifier: Sparkles,
    objective: Target,
};

const KIND_LABEL: Record<ChallengeKind, string> = {
    restriction: "Restriction",
    modifier: "Modifier",
    objective: "Objective",
};

export function ModifierSlab({ challenge, onReroll }: IModifierSlabProps): React.ReactElement {
    const Icon = KIND_ICON[challenge.kind];

    return (
        <SlabFrame index="03" kicker="RULE" accent="palm">
            <div className="flex items-start justify-between gap-3">
                <div className="flex min-w-0 flex-1 items-center gap-2 text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                    <Icon aria-hidden="true" className="h-3.5 w-3.5" />
                    <span>{KIND_LABEL[challenge.kind]}</span>
                </div>
                <Button onClick={onReroll} size="xs" variant="outline" aria-label="Reroll modifier">
                    <RotateCcw aria-hidden="true" />
                    Reroll
                </Button>
            </div>

            <div className="mt-3">
                <h3 className="m-0 font-bold font-[var(--font-display)] text-[28px] leading-tight tracking-tight text-foreground sm:text-[34px]">{challenge.title}</h3>
                <p className="mt-2 max-w-prose text-sm text-muted-foreground sm:text-base">{challenge.description}</p>
            </div>
        </SlabFrame>
    );
}
