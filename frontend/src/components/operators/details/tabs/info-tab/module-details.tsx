"use client";

import { useState } from "react";
import { ChevronDown } from "lucide-react";
import { motion } from "motion/react";
import Image from "next/image";
import type { Module } from "~/types/api";
import { Disclosure, DisclosureTrigger, DisclosureContent } from "~/components/ui/motion-primitives/disclosure";

interface ModuleDetailsProps {
    modules: Module[];
}

export function ModuleDetails({ modules }: ModuleDetailsProps) {
    const [isExpanded, setIsExpanded] = useState(true);

    // Filter out initial type modules
    const displayModules = modules.filter((m) => m.type !== "INITIAL");

    if (displayModules.length === 0) return null;

    return (
        <Disclosure open={isExpanded} onOpenChange={setIsExpanded}>
            <DisclosureTrigger>
                <div className="flex cursor-pointer items-center justify-between rounded-lg bg-muted/30 p-4 transition-colors hover:bg-muted/50">
                    <h3 className="text-lg font-semibold">Module Details</h3>
                    <motion.div animate={{ rotate: isExpanded ? 180 : 0 }} transition={{ duration: 0.2 }}>
                        <ChevronDown className="h-5 w-5" />
                    </motion.div>
                </div>
            </DisclosureTrigger>
            <DisclosureContent>
                <div className="mt-2 space-y-3">
                    {displayModules.map((module) => (
                        <div key={module.uniEquipId} className="rounded-lg border border-border bg-card/50 p-4">
                            <div className="flex items-start gap-3">
                                {module.image && (
                                    <div className="relative h-16 w-16 flex-shrink-0 overflow-hidden rounded-md bg-muted">
                                        <Image src={module.image || "/placeholder.svg"} alt={module.uniEquipName} fill className="object-contain p-1" />
                                    </div>
                                )}
                                <div className="flex-1">
                                    <div className="flex items-center gap-2">
                                        <span className="text-xs font-semibold text-primary">
                                            {module.typeName1}
                                            {module.typeName2 && ` - ${module.typeName2}`}
                                        </span>
                                    </div>
                                    <h4 className="font-semibold">{module.uniEquipName}</h4>
                                    <p className="mt-1 text-sm text-muted-foreground">{module.uniEquipDesc}</p>
                                </div>
                            </div>
                        </div>
                    ))}
                </div>
            </DisclosureContent>
        </Disclosure>
    );
}
