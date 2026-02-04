"use client";

import { useState } from "react";
import { cn } from "~/lib/utils";
import type { Operator } from "~/types/api";
import { MaterialItem } from "./material-item";

interface SkillMasteryTabProps {
    operator: Operator;
}

export function SkillMasteryTab({ operator }: SkillMasteryTabProps) {
    const [selectedSkill, setSelectedSkill] = useState(0);

    if (!operator.skills || operator.skills.length === 0) {
        return <div className="py-8 text-center text-muted-foreground">No skills available for this operator.</div>;
    }

    const skill = operator.skills[selectedSkill];
    const levelUpCosts = skill?.levelUpCostCond ?? [];

    return (
        <div className="space-y-4">
            {/* Skill selector */}
            <div className="flex flex-wrap gap-2">
                {operator.skills.map((s, idx) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: Static skill list
                    <button className={cn("rounded-lg border px-4 py-2 font-medium text-sm transition-all", selectedSkill === idx ? "border-primary bg-primary/10 text-primary" : "border-border bg-secondary/30 text-muted-foreground hover:text-foreground")} key={idx} onClick={() => setSelectedSkill(idx)} type="button">
                        {s.static?.Levels?.[0]?.name ?? `Skill ${idx + 1}`}
                    </button>
                ))}
            </div>

            {/* Mastery costs */}
            <div className="space-y-3">
                {levelUpCosts.map((costData, idx) => {
                    if (!costData.LevelUpCost || costData.LevelUpCost.length === 0) return null;
                    return (
                        // biome-ignore lint/suspicious/noArrayIndexKey: Static mastery level list
                        <div className="rounded-lg border border-border bg-card/30 p-4" key={idx}>
                            <div className="mb-3 flex items-center gap-2">
                                <span className="rounded bg-primary/20 px-2 py-0.5 font-semibold text-primary text-sm">M{idx + 1}</span>
                            </div>
                            <div className="flex flex-wrap gap-4">
                                {costData.LevelUpCost.map((cost, costIdx) => (
                                    // biome-ignore lint/suspicious/noArrayIndexKey: Static cost list
                                    <MaterialItem count={cost.Count} id={cost.Id} image={cost.Image} key={costIdx} />
                                ))}
                            </div>
                        </div>
                    );
                })}
            </div>

            {levelUpCosts.length === 0 && <div className="py-8 text-center text-muted-foreground">No mastery data available for this skill.</div>}
        </div>
    );
}
