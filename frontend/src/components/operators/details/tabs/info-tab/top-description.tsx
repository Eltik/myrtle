"use client";

import { insertBlackboard } from "~/lib/operator-helpers";
import type { Operator } from "~/types/api";

interface TopDescriptionProps {
    operator: Operator;
}

export function TopDescription({ operator }: TopDescriptionProps) {
    // Get trait description with blackboard values
    const getTraitDescription = () => {
        const trait = operator.trait;
        if (!trait?.Candidates?.length) return operator.description;

        const candidate = trait.Candidates[trait.Candidates.length - 1];
        if (!candidate?.OverrideDescription) return operator.description;

        const formatted = insertBlackboard(candidate.OverrideDescription, candidate.Blackboard);
        return formatted ?? operator.description;
    };

    const traitDescription = getTraitDescription();

    return (
        <div className="rounded-lg border border-border bg-muted/30 p-4">
            <h4 className="mb-2 font-semibold text-muted-foreground text-sm uppercase tracking-wide">Trait</h4>
            <p className="text-foreground text-sm leading-relaxed" dangerouslySetInnerHTML={{ __html: traitDescription ?? "" }} />
        </div>
    );
}
