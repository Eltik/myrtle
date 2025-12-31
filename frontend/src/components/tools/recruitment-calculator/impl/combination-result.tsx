"use client";

import { ChevronDown } from "lucide-react";
import { useTheme } from "next-themes";
import { useState } from "react";
import { RARITY_COLORS, RARITY_COLORS_LIGHT } from "~/components/operators/list/constants";
import { Disclosure, DisclosureContent, DisclosureTrigger } from "~/components/ui/motion-primitives/disclosure";
import { cn } from "~/lib/utils";
import { SENIOR_OPERATOR_TAG_ID, TOP_OPERATOR_TAG_ID } from "./constants";
import { OperatorResultCard } from "./operator-result-card";
import type { TagCombinationResult } from "./types";

interface CombinationResultProps {
    result: TagCombinationResult;
    defaultExpanded?: boolean;
}

export function CombinationResult({ result, defaultExpanded = false }: CombinationResultProps) {
    const [isOpen, setIsOpen] = useState(defaultExpanded);
    const { resolvedTheme } = useTheme();

    const rarityColors = resolvedTheme === "light" ? RARITY_COLORS_LIGHT : RARITY_COLORS;
    const guaranteedColor = rarityColors[result.guaranteedRarity] ?? "#ffffff";
    const _isHighValue = result.guaranteedRarity >= 5;
    const hasTopOperator = result.tags.includes(TOP_OPERATOR_TAG_ID);
    const hasSeniorOperator = result.tags.includes(SENIOR_OPERATOR_TAG_ID);

    return (
        <div
            className={cn(
                "overflow-hidden rounded-lg border",
                result.guaranteedRarity === 6 ? "border-orange-500/40 bg-orange-500/10 dark:border-orange-500/30 dark:bg-orange-500/15" : result.guaranteedRarity === 5 ? "border-yellow-500/40 bg-yellow-500/10 dark:border-yellow-500/25 dark:bg-yellow-500/12" : "border-border bg-card/50",
            )}
        >
            <Disclosure onOpenChange={setIsOpen} open={isOpen}>
                <DisclosureTrigger>
                    <div className="flex w-full items-center justify-between p-3 text-left hover:bg-muted/30 data-[state=open]:bg-muted/30 sm:p-4">
                        <div className="flex flex-wrap items-center gap-2">
                            {/* Guaranteed rarity badge */}
                            <span className={cn("flex items-center gap-1 rounded-md px-2 py-0.5 font-bold text-xs", result.guaranteedRarity === 6 ? "bg-orange-500/20 dark:bg-orange-500/25" : result.guaranteedRarity === 5 ? "bg-yellow-500/20 dark:bg-yellow-500/22" : "bg-muted")} style={{ color: guaranteedColor }}>
                                {result.guaranteedRarity}★{result.guaranteedRarity < result.maxRarity && `~${result.maxRarity}★`}
                            </span>

                            {/* Tags */}
                            <div className="flex flex-wrap gap-1">
                                {result.tagNames.map((tagName, idx) => {
                                    const tagId = result.tags[idx];
                                    return (
                                        <span
                                            className={cn(
                                                "rounded px-1.5 py-0.5 text-xs",
                                                hasTopOperator && tagId === TOP_OPERATOR_TAG_ID
                                                    ? "bg-amber-500/15 text-amber-700 dark:bg-amber-500/20 dark:text-amber-400"
                                                    : hasSeniorOperator && tagId === SENIOR_OPERATOR_TAG_ID
                                                      ? "bg-amber-500/15 text-amber-700 dark:bg-amber-500/20 dark:text-amber-400"
                                                      : "bg-muted text-muted-foreground",
                                            )}
                                            key={tagId}
                                        >
                                            {tagName}
                                        </span>
                                    );
                                })}
                            </div>

                            {/* Operator count */}
                            <span className="text-muted-foreground text-xs">
                                ({result.operators.length} operator{result.operators.length !== 1 ? "s" : ""})
                            </span>
                        </div>

                        <ChevronDown className="h-4 w-4 text-muted-foreground transition-transform duration-200 group-data-[state=open]:rotate-180" />
                    </div>
                </DisclosureTrigger>

                <DisclosureContent>
                    <div className="border-border/50 border-t p-3 sm:p-4">
                        <div className="grid grid-cols-3 gap-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-7 xl:grid-cols-8">
                            {result.operators.map((op) => (
                                <OperatorResultCard key={op.id} operator={op} />
                            ))}
                        </div>
                    </div>
                </DisclosureContent>
            </Disclosure>
        </div>
    );
}
