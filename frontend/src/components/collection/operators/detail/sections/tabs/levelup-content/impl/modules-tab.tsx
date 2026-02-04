"use client";

import Image from "next/image";
import type { Operator } from "~/types/api";
import { formatStatKey, formatStatValue } from "./helpers";
import { MaterialItem } from "./material-item";

interface ModulesTabProps {
    operator: Operator;
}

export function ModulesTab({ operator }: ModulesTabProps) {
    if (!operator.modules || operator.modules.length === 0) {
        return <div className="py-8 text-center text-muted-foreground">This operator has no modules.</div>;
    }

    return (
        <div className="space-y-4">
            {operator.modules.map((mod) => (
                <div className="rounded-lg border border-border bg-card/30 p-4" key={mod.uniEquipId}>
                    {/* Module Header */}
                    <div className="mb-4 flex items-center gap-3">
                        {mod.image && <Image alt={mod.uniEquipName ?? "Module"} className="rounded object-contain" height={48} src={`/api/cdn${mod.image}`} width={48} />}
                        <div className="flex-1">
                            <div className="flex items-center gap-2">
                                <h4 className="font-medium text-foreground">{mod.uniEquipName ?? "Module"}</h4>
                                <span className="rounded bg-secondary px-1.5 py-0.5 text-muted-foreground text-xs">{mod.typeName1}</span>
                            </div>
                            <p className="text-muted-foreground text-xs">
                                Unlock: E{mod.unlockEvolvePhase} Lv.{mod.unlockLevel}
                                {mod.unlockFavorPoint > 0 && ` · Trust ${Math.floor(mod.unlockFavorPoint / 100)}%`}
                            </p>
                        </div>
                    </div>

                    {/* Module Stages */}
                    {mod.data?.phases && mod.data.phases.length > 0 && (
                        <div className="space-y-3">
                            {mod.data.phases.map((phase, phaseIdx) => {
                                // Get costs for this specific stage from itemCost (keys are "1", "2", "3")
                                const stageKey = String(phaseIdx + 1);
                                const stageCosts = mod.itemCost?.[stageKey] ?? [];

                                return (
                                    <div className="rounded border border-border/50 bg-secondary/20 p-3" key={phase.equipLevel}>
                                        <div className="mb-2 flex items-center gap-2">
                                            <span className="rounded bg-primary/20 px-2 py-0.5 font-semibold text-primary text-sm">Stage {phaseIdx + 1}</span>
                                        </div>

                                        {/* Materials for this stage */}
                                        {stageCosts.length > 0 ? (
                                            <div className="flex flex-wrap gap-4">
                                                {stageCosts.map((cost, costIdx) => (
                                                    // biome-ignore lint/suspicious/noArrayIndexKey: Static cost list
                                                    <MaterialItem count={cost.count} id={cost.id} image={cost.image} key={costIdx} size="sm" />
                                                ))}
                                            </div>
                                        ) : (
                                            <p className="text-muted-foreground text-xs italic">No material data available</p>
                                        )}

                                        {/* Attribute bonuses for this stage */}
                                        {phase.attributeBlackboard && phase.attributeBlackboard.length > 0 && (
                                            <div className="mt-3 flex flex-wrap gap-2">
                                                {phase.attributeBlackboard.map((attr, attrIdx) => (
                                                    // biome-ignore lint/suspicious/noArrayIndexKey: Static attribute list
                                                    <span className="rounded bg-primary/10 px-2 py-1 text-primary text-xs" key={attrIdx}>
                                                        {formatStatKey(attr.key)}: {formatStatValue(attr.value)}
                                                    </span>
                                                ))}
                                            </div>
                                        )}
                                    </div>
                                );
                            })}
                        </div>
                    )}
                </div>
            ))}
        </div>
    );
}
