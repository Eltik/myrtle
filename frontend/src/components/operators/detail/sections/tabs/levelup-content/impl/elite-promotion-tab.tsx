"use client";

import { ChevronRight } from "lucide-react";
import Image from "next/image";
import type { Operator } from "~/types/api";
import { MaterialItem } from "./material-item";

interface ElitePromotionTabProps {
    operator: Operator;
}

export function ElitePromotionTab({ operator }: ElitePromotionTabProps) {
    return (
        <div className="grid gap-4 md:grid-cols-2">
            {operator.phases.map((phase, idx) => {
                if (idx === 0 || !phase.EvolveCost || phase.EvolveCost.length === 0) return null;

                return (
                    // biome-ignore lint/suspicious/noArrayIndexKey: Static promotion phases
                    <div className="rounded-lg border border-border bg-card/30 p-4" key={idx}>
                        <div className="mb-3 flex items-center gap-3">
                            <div className="flex items-center gap-2">
                                <Image alt={`E${idx - 1}`} height={24} src={`/api/cdn/upk/arts/elite_hub/elite_${idx - 1}.png`} width={24} />
                                <ChevronRight className="h-4 w-4 text-muted-foreground" />
                                <Image alt={`E${idx}`} height={24} src={`/api/cdn/upk/arts/elite_hub/elite_${idx}.png`} width={24} />
                            </div>
                            <span className="font-medium text-foreground">Elite {idx}</span>
                        </div>

                        <div className="flex flex-wrap gap-4">
                            {phase.EvolveCost.map((cost, costIdx) => (
                                // biome-ignore lint/suspicious/noArrayIndexKey: Static cost list
                                <MaterialItem count={cost.Count} id={cost.Id} image={cost.Image} key={costIdx} />
                            ))}
                        </div>
                    </div>
                );
            })}

            {operator.phases.length <= 1 && <div className="py-8 text-center text-muted-foreground md:col-span-2">This operator cannot be promoted.</div>}
        </div>
    );
}
