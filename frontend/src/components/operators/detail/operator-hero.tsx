"use client";

import Image from "next/image";
import Link from "next/link";
import { motion } from "motion/react";
import { ChevronLeft } from "lucide-react";
import type { Operator } from "~/types/api";
import { RarityStars } from "~/components/operators/list/ui/impl/rarity-stars";
import { ClassIcon } from "~/components/operators/list/ui/impl/class-icon";
import { CLASS_DISPLAY, NATION_DISPLAY } from "~/components/operators/list/constants";
import { cn } from "~/lib/utils";

interface OperatorHeroProps {
    operator: Operator;
}

const RARITY_MAP: Record<string, number> = {
    TIER_6: 6,
    TIER_5: 5,
    TIER_4: 4,
    TIER_3: 3,
    TIER_2: 2,
    TIER_1: 1,
};

export function OperatorHero({ operator }: OperatorHeroProps) {
    const rarityNum = RARITY_MAP[operator.rarity] ?? 1;
    const skinSuffix = (operator.phases?.length ?? 0) > 2 ? "2b" : "1b";
    const imageUrl = `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/skin/${operator.id ?? ""}_${skinSuffix}.png`;
    const classDisplay = CLASS_DISPLAY[operator.profession] ?? operator.profession;
    const nationDisplay = NATION_DISPLAY[operator.nationId ?? ""] ?? operator.nationId;

    return (
        <div className="relative mb-8 h-[500px] w-full overflow-hidden md:h-[600px]">
            {/* Background image with gradient overlays */}
            <div className="absolute inset-0">
                <Image src={imageUrl || "/placeholder.svg"} alt={operator.name} fill className="object-cover object-top" priority unoptimized />
                {/* Multiple gradient overlays for depth */}
                <div className="absolute inset-0 bg-gradient-to-t from-background via-background/60 to-transparent" />
                <div className="absolute inset-0 bg-gradient-to-r from-background/80 via-transparent to-background/80" />
            </div>

            {/* Content overlay */}
            <div className="relative z-10 flex h-full flex-col justify-end px-4 pb-8 md:px-8">
                <div className="mx-auto w-full max-w-6xl">
                    {/* Breadcrumb */}
                    <motion.div initial={{ opacity: 0, x: -20 }} animate={{ opacity: 1, x: 0 }} transition={{ delay: 0.1 }}>
                        <Link href="/operators/list" className="mb-4 inline-flex items-center gap-1 text-sm text-muted-foreground transition-colors hover:text-foreground">
                            <ChevronLeft className="h-4 w-4" />
                            Back to Operators
                        </Link>
                    </motion.div>

                    {/* Operator name and basic info */}
                    <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: 0.2 }} className="space-y-3">
                        <div className="flex items-center gap-3">
                            <RarityStars rarity={rarityNum} className="flex" starClassName="text-lg" />
                        </div>

                        <h1 className="text-4xl font-bold tracking-tight text-foreground md:text-6xl">{operator.name}</h1>

                        <div className="flex flex-wrap items-center gap-4 text-sm text-muted-foreground">
                            <div className="flex items-center gap-2">
                                <ClassIcon profession={operator.profession} size={20} className="opacity-80" />
                                <span>{classDisplay}</span>
                            </div>
                            {nationDisplay && (
                                <div className="flex items-center gap-2">
                                    <span className="text-muted-foreground/60">|</span>
                                    <span>{nationDisplay}</span>
                                </div>
                            )}
                            <div className="flex items-center gap-2">
                                <span className="text-muted-foreground/60">|</span>
                                <span className={cn("rounded-full px-2 py-0.5 text-xs font-medium", operator.position === "MELEE" ? "bg-orange-500/20 text-orange-400" : "bg-blue-500/20 text-blue-400")}>{operator.position}</span>
                            </div>
                        </div>

                        {operator.itemUsage && <p className="max-w-2xl text-sm leading-relaxed text-muted-foreground">{operator.itemUsage}</p>}
                    </motion.div>
                </div>
            </div>
        </div>
    );
}
