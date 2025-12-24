"use client";

import { motion } from "motion/react";
import { ChevronDown } from "lucide-react";
import { useState } from "react";
import { cn } from "~/lib/utils";
import { RARITY_COLORS } from "~/components/operators/list/constants";
import { TOP_OPERATOR_TAG_ID, SENIOR_OPERATOR_TAG_ID } from "./constants";
import type { TagCombinationResult } from "./types";
import { OperatorResultCard } from "./operator-result-card";

interface CombinationResultProps {
    result: TagCombinationResult;
    defaultExpanded?: boolean;
}

export function CombinationResult({ result, defaultExpanded = false }: CombinationResultProps) {
    const [isExpanded, setIsExpanded] = useState(defaultExpanded);
    const guaranteedColor = RARITY_COLORS[result.guaranteedRarity] ?? "#ffffff";
    const isHighValue = result.guaranteedRarity >= 5;
    const hasTopOperator = result.tags.includes(TOP_OPERATOR_TAG_ID);
    const hasSeniorOperator = result.tags.includes(SENIOR_OPERATOR_TAG_ID);

    return (
        <div className={cn("overflow-hidden rounded-lg border transition-colors", isHighValue ? "border-amber-500/30 bg-amber-500/5" : "border-border bg-card/50", isExpanded && "ring-1 ring-primary/20")}>
            {/* Header */}
            <button className="flex w-full items-center justify-between p-3 text-left transition-colors hover:bg-muted/30 sm:p-4" onClick={() => setIsExpanded(!isExpanded)} type="button">
                <div className="flex flex-wrap items-center gap-2">
                    {/* Guaranteed rarity badge */}
                    <span className={cn("flex items-center gap-1 rounded-md px-2 py-0.5 font-bold text-xs", isHighValue ? "bg-amber-500/20" : "bg-muted")} style={{ color: guaranteedColor }}>
                        {result.guaranteedRarity}★{result.guaranteedRarity < result.maxRarity && `~${result.maxRarity}★`}
                    </span>

                    {/* Tags */}
                    <div className="flex flex-wrap gap-1">
                        {result.tagNames.map((tagName, idx) => {
                            const tagId = result.tags[idx];
                            return (
                                <span className={cn("rounded px-1.5 py-0.5 text-xs", hasTopOperator && tagId === TOP_OPERATOR_TAG_ID ? "bg-amber-500/20 text-amber-400" : hasSeniorOperator && tagId === SENIOR_OPERATOR_TAG_ID ? "bg-amber-500/20 text-amber-400" : "bg-muted text-muted-foreground")} key={tagId}>
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

                <ChevronDown className={cn("h-4 w-4 text-muted-foreground transition-transform", isExpanded && "rotate-180")} />
            </button>

            {/* Expanded content */}
            <motion.div animate={{ height: isExpanded ? "auto" : 0 }} className="overflow-hidden" initial={false} transition={{ duration: 0.2 }}>
                <div className="border-border/50 border-t p-3 sm:p-4">
                    <div className="grid grid-cols-4 gap-3 sm:grid-cols-5 md:grid-cols-6 lg:grid-cols-8">
                        {result.operators.map((op) => (
                            <OperatorResultCard key={op.id} operator={op} />
                        ))}
                    </div>
                </div>
            </motion.div>
        </div>
    );
}
