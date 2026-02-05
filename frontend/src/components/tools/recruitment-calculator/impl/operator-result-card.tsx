"use client";

import { motion } from "motion/react";
import Image from "next/image";
import { CLASS_DISPLAY, CLASS_ICON, RARITY_COLORS } from "~/components/collection/operators/list/constants";
import { RarityStars } from "~/components/collection/operators/list/ui/impl/rarity-stars";
import { cn } from "~/lib/utils";
import type { RecruitableOperator } from "~/types/frontend/impl/tools/recruitment";

interface OperatorResultCardProps {
    operator: RecruitableOperator;
}

export function OperatorResultCard({ operator }: OperatorResultCardProps) {
    const rarityColor = RARITY_COLORS[operator.rarity] ?? "#ffffff";
    const professionDisplay = CLASS_DISPLAY[operator.profession] ?? operator.profession;
    const professionIcon = CLASS_ICON[operator.profession] ?? operator.profession.toLowerCase();

    return (
        <motion.div className="group card-hover-transition relative flex aspect-2/3 overflow-clip rounded-md border border-muted/50 bg-card contain-content" whileHover={{ scale: 1.05 }}>
            {/* Operator portrait */}
            <div className={cn("absolute inset-0 origin-center transform-gpu transition-all duration-200 ease-out")}>
                <Image alt={operator.name} className="h-full w-full rounded-lg object-cover" height={128} src={`/api/cdn/portrait/${operator.id}`} unoptimized width={128} />
            </div>

            {/* Bottom info bar */}
            <div className="absolute inset-x-0 bottom-0 z-10">
                <div className="relative">
                    {/* Gradient overlay */}
                    <div className="h-24 w-full bg-linear-to-t from-black/90 via-black/60 to-transparent" />

                    {/* Content */}
                    <div className="absolute inset-0 flex flex-col justify-end p-2">
                        {/* Name */}
                        <span className="truncate font-bold text-sm text-white">{operator.name}</span>

                        <div className="flex items-center justify-between">
                            {/* Rarity stars */}
                            <RarityStars className="flex" rarity={operator.rarity} starClassName="text-2.5" />

                            {/* Class icon */}
                            <div className="h-5 w-5">
                                <Image alt={professionDisplay} height={20} src={`/api/cdn/upk/arts/ui/[uc]charcommon/icon_profession_${professionIcon}.png`} width={20} />
                            </div>
                        </div>
                    </div>

                    {/* Rarity color bar */}
                    <div className="h-0.5 w-full" style={{ backgroundColor: rarityColor }} />
                </div>
            </div>
        </motion.div>
    );
}
