"use client";

import Image from "next/image";
import { RARITY_COLORS, CLASS_DISPLAY, CLASS_ICON } from "~/components/operators/list/constants";
import { RarityStars } from "~/components/operators/list/ui/impl/rarity-stars";
import { cn } from "~/lib/utils";
import type { RecruitableOperator } from "./types";
import { Tooltip, TooltipContent, TooltipTrigger } from "~/components/ui/shadcn/tooltip";

interface OperatorResultCardProps {
    operator: RecruitableOperator;
}

export function OperatorResultCard({ operator }: OperatorResultCardProps) {
    const rarityColor = RARITY_COLORS[operator.rarity] ?? "#ffffff";
    const professionDisplay = CLASS_DISPLAY[operator.profession] ?? operator.profession;
    const professionIcon = CLASS_ICON[operator.profession] ?? operator.profession.toLowerCase();

    return (
        <Tooltip>
            <TooltipTrigger asChild>
                <div className="group relative flex flex-col items-center">
                    {/* Avatar container */}
                    <div className={cn("relative h-14 w-14 overflow-hidden rounded-lg border border-border/50 bg-card transition-all sm:h-16 sm:w-16", "group-hover:border-primary/50 group-hover:shadow-lg group-hover:shadow-primary/10")}>
                        {/* Rarity glow */}
                        <div className="absolute inset-0 opacity-20" style={{ background: `radial-gradient(circle at center, ${rarityColor} 0%, transparent 70%)` }} />

                        {/* Operator portrait */}
                        <Image alt={operator.name} className="h-full w-full object-cover transition-transform group-hover:scale-110" height={64} src={`/api/cdn/portrait/${operator.id}`} width={64} />

                        {/* Class icon */}
                        <div className="absolute right-0.5 bottom-0.5 h-4 w-4 opacity-70">
                            <Image alt={professionDisplay} height={16} src={`/api/cdn/upk/arts/ui/[uc]charcommon/icon_profession_${professionIcon}.png`} width={16} />
                        </div>

                        {/* Rarity bar */}
                        <div className="absolute inset-x-0 bottom-0 h-0.5" style={{ backgroundColor: rarityColor }} />
                    </div>

                    {/* Name */}
                    <span className="mt-1 max-w-16 truncate text-center font-medium text-[10px] text-muted-foreground transition-colors group-hover:text-foreground sm:text-xs">{operator.name}</span>

                    {/* Rarity stars */}
                    <RarityStars className="flex" rarity={operator.rarity} starClassName="text-[8px] sm:text-[10px]" />
                </div>
            </TooltipTrigger>
            <TooltipContent>
                <div className="space-y-1">
                    <p className="font-semibold">{operator.name}</p>
                    <p className="text-xs" style={{ color: rarityColor }}>
                        {operator.rarity}★ {professionDisplay}
                    </p>
                </div>
            </TooltipContent>
        </Tooltip>
    );
}
