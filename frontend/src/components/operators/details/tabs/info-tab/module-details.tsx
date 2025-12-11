"use client";

import { ChevronDown } from "lucide-react";
import { motion } from "motion/react";
import Image from "next/image";
import { useState } from "react";
import { Disclosure, DisclosureContent, DisclosureTrigger } from "~/components/ui/motion-primitives/disclosure";
import type { Module } from "~/types/api";

interface ModuleDetailsProps {
    modules: Module[];
}

export function ModuleDetails({ modules }: ModuleDetailsProps) {
    const [isExpanded, setIsExpanded] = useState(true);

    // Filter out initial type modules
    const displayModules = modules.filter((m) => m.type !== "INITIAL");

    if (displayModules.length === 0) return null;

    return (
        <Disclosure onOpenChange={setIsExpanded} open={isExpanded}>
            <DisclosureTrigger>
                <div className="flex cursor-pointer items-center justify-between rounded-lg bg-muted/30 p-4 transition-colors hover:bg-muted/50">
                    <h3 className="font-semibold text-lg">Module Details</h3>
                    <motion.div animate={{ rotate: isExpanded ? 180 : 0 }} transition={{ duration: 0.2 }}>
                        <ChevronDown className="h-5 w-5" />
                    </motion.div>
                </div>
            </DisclosureTrigger>
            <DisclosureContent>
                <div className="mt-2 space-y-3">
                    {displayModules.map((module) => (
                        <div className="rounded-lg border border-border bg-card/50 p-4" key={module.uniEquipId}>
                            <div className="flex items-start gap-3">
                                {module.image && (
                                    <div className="relative h-16 w-16 flex-shrink-0 overflow-hidden rounded-md bg-muted">
                                        <Image alt={module.uniEquipName} className="object-contain p-1" fill src={module.image || "/placeholder.svg"} />
                                    </div>
                                )}
                                <div className="flex-1">
                                    <div className="flex items-center gap-2">
                                        <span className="font-semibold text-primary text-xs">
                                            {module.typeName1}
                                            {module.typeName2 && ` - ${module.typeName2}`}
                                        </span>
                                    </div>
                                    <h4 className="font-semibold">{module.uniEquipName}</h4>
                                    <p className="mt-1 text-muted-foreground text-sm">{module.uniEquipDesc}</p>
                                </div>
                            </div>
                        </div>
                    ))}
                </div>
            </DisclosureContent>
        </Disclosure>
    );
}
