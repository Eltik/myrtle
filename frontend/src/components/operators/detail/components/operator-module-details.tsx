"use client";

import type { OperatorModule } from "~/types/api";
import { cn } from "~/lib/utils";

interface OperatorModuleDetailsProps {
    modules: OperatorModule[];
    currentModule: string;
}

export function OperatorModuleDetails({ modules, currentModule }: OperatorModuleDetailsProps) {
    if (!modules || modules.length === 0) {
        return <p className="py-4 text-sm text-muted-foreground">No modules available for this operator.</p>;
    }

    const activeModule = modules.find((m) => m.uniEquipId === currentModule) ?? modules[0];

    return (
        <div className="space-y-4 pt-4">
            {/* Module tabs */}
            <div className="flex flex-wrap gap-2">
                {modules.map((mod) => (
                    <div key={mod.uniEquipId} className={cn("rounded-lg border px-4 py-2 text-sm transition-colors", mod.uniEquipId === activeModule?.uniEquipId ? "border-primary bg-primary/10" : "border-border bg-muted/50")}>
                        <p className="font-medium">{mod.uniEquipName ?? mod.uniEquipId}</p>
                        <p className="text-xs text-muted-foreground">{mod.typeIcon}</p>
                    </div>
                ))}
            </div>

            {/* Module description */}
            {activeModule && (
                <div className="space-y-3">
                    {activeModule.uniEquipDesc && (
                        <div className="rounded-lg bg-muted/50 p-4">
                            <h4 className="mb-2 text-sm font-medium">Description</h4>
                            <p className="text-sm text-muted-foreground">{activeModule.uniEquipDesc}</p>
                        </div>
                    )}
                </div>
            )}
        </div>
    );
}
