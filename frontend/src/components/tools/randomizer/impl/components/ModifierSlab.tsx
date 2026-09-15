import { Flag, RotateCcw, Sparkles, Target } from "lucide-react";
import type React from "react";
import { Button } from "#/components/ui/button";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages as challengeMessages } from "../challenges.messages";
import type { ChallengeKind, IChallenge } from "../types";
import type { messages } from "./ModifierSlab.messages";
import { SlabFrame } from "./SlabFrame";

/** This panel renders its own chrome plus the challenge text from `challenges.ts`. */
type ModifierT = TypedT<typeof messages & typeof challengeMessages>;

interface IModifierSlabProps {
    challenge: IChallenge;
    onReroll: () => void;
}

const KIND_ICON: Record<ChallengeKind, typeof Flag> = {
    restriction: Flag,
    modifier: Sparkles,
    objective: Target,
};

const KIND_LABEL_KEY: Record<ChallengeKind, keyof typeof messages & string> = {
    restriction: "randomizer.modifier.kind.restriction",
    modifier: "randomizer.modifier.kind.modifier",
    objective: "randomizer.modifier.kind.objective",
};

export function ModifierSlab({ challenge, onReroll }: IModifierSlabProps): React.ReactElement {
    const t: ModifierT = useT("tools");
    const Icon = KIND_ICON[challenge.kind];

    return (
        <SlabFrame index="03" kicker={t("randomizer.modifier.kicker")} accent="palm">
            <div className="flex items-start justify-between gap-3">
                <div className="flex min-w-0 flex-1 items-center gap-2 text-[11px] text-muted-foreground uppercase tracking-[0.18em]">
                    <Icon aria-hidden="true" className="h-3.5 w-3.5" />
                    <span>{t(KIND_LABEL_KEY[challenge.kind])}</span>
                </div>
                <Button onClick={onReroll} size="xs" variant="outline" aria-label={t("randomizer.modifier.rerollAria")}>
                    <RotateCcw aria-hidden="true" />
                    {t("randomizer.modifier.reroll")}
                </Button>
            </div>

            <div className="mt-3">
                <h3 className="m-0 font-semibold text-[28px] text-foreground leading-tight tracking-tight sm:text-[34px]">{t(challenge.titleKey)}</h3>
                <p className="mt-2 max-w-prose text-muted-foreground text-sm sm:text-base">{t(challenge.descKey)}</p>
            </div>
        </SlabFrame>
    );
}
