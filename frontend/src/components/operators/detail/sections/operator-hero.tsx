"use client";

import { motion } from "motion/react";
import Image from "next/image";
import Link from "next/link";
import { ChevronRight } from "lucide-react";
import type { Operator } from "~/types/api";
import { formatProfession, formatSubProfession, rarityToNumber } from "~/lib/utils";
import { cn, formatNationId } from "~/lib/utils";
import { Badge } from "~/components/ui/shadcn/badge";

interface OperatorHeroProps {
    operator: Operator;
}

const RARITY_COLORS: Record<string, string> = {
    TIER_6: "text-[#f7a452] border-[#f7a452]/50",
    TIER_5: "text-[#f7e79e] border-[#f7e79e]/50",
    TIER_4: "text-[#bcabdb] border-[#bcabdb]/50",
    TIER_3: "text-[#88c8e3] border-[#88c8e3]/50",
    TIER_2: "text-[#7ef2a3] border-[#7ef2a3]/50",
    TIER_1: "text-white border-white/50",
};

const RARITY_BG_COLORS: Record<string, string> = {
    TIER_6: "bg-[#f7a452]/10",
    TIER_5: "bg-[#f7e79e]/10",
    TIER_4: "bg-[#bcabdb]/10",
    TIER_3: "bg-[#88c8e3]/10",
    TIER_2: "bg-[#7ef2a3]/10",
    TIER_1: "bg-white/10",
};

export function OperatorHero({ operator }: OperatorHeroProps) {
    const rarityNum = rarityToNumber(operator.rarity);
    const rarityColor = RARITY_COLORS[operator.rarity] ?? RARITY_COLORS.TIER_1;
    const rarityBgColor = RARITY_BG_COLORS[operator.rarity] ?? RARITY_BG_COLORS.TIER_1;

    // Determine the skin image - use E2 art if available
    const operatorId = operator.id ?? "";
    const artSuffix = operator.phases.length > 2 ? "_2" : "_1";
    const portraitUrl = `/api/cdn/upk/chararts/${operatorId}/${operatorId}${artSuffix}.png`;

    return (
        <div className="relative h-[500px] w-full overflow-hidden md:h-[600px]">
            {/* Background Image */}
            <div className="absolute inset-0">
                <Image alt={operator.name} className="object-cover object-top opacity-40" fill priority src={portraitUrl || "/placeholder.svg"} />
                {/* Gradient overlays */}
                <div className="absolute inset-0 bg-linear-to-t from-background via-background/80 to-transparent" />
                <div className="absolute inset-0 bg-linear-to-r from-background/60 via-transparent to-background/60" />
            </div>

            {/* Content */}
            <div className="relative z-10 mx-auto flex h-full max-w-6xl flex-col justify-end px-4 pb-8 md:px-8">
                {/* Breadcrumb */}
                <motion.nav animate={{ opacity: 1, y: 0 }} className="mb-4" initial={{ opacity: 0, y: -10 }} transition={{ duration: 0.3 }}>
                    <ol className="flex items-center gap-2 text-muted-foreground text-sm">
                        <li>
                            <Link className="transition-colors hover:text-foreground" href="/operators/list">
                                Operators
                            </Link>
                        </li>
                        <ChevronRight className="h-4 w-4" />
                        <li className="text-foreground">{operator.name}</li>
                    </ol>
                </motion.nav>

                {/* Operator Info */}
                <div className="flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
                    <div className="flex flex-col gap-3">
                        {/* Name and Rarity */}
                        <motion.div animate={{ opacity: 1, y: 0 }} initial={{ opacity: 0, y: 20 }} transition={{ duration: 0.4, delay: 0.1 }}>
                            <h1 className="font-bold text-4xl text-foreground md:text-5xl lg:text-6xl">{operator.name}</h1>
                            <div className="mt-2 flex items-center gap-3">
                                <span className={cn("font-semibold text-lg", rarityColor)}>{Array(rarityNum).fill("★").join("")}</span>
                                <span className="text-muted-foreground">|</span>
                                <span className="text-muted-foreground">
                                    {formatSubProfession(operator.subProfessionId)} {formatProfession(operator.profession)}
                                </span>
                            </div>
                        </motion.div>

                        {/* Tags */}
                        <motion.div animate={{ opacity: 1, y: 0 }} className="flex flex-wrap gap-2" initial={{ opacity: 0, y: 20 }} transition={{ duration: 0.4, delay: 0.2 }}>
                            <Badge className={cn("border", rarityColor, rarityBgColor)} variant="outline">
                                {formatProfession(operator.profession)}
                            </Badge>
                            <Badge className="border-border bg-secondary/50" variant="outline">
                                {operator.position === "RANGED" ? "Ranged" : operator.position === "MELEE" ? "Melee" : operator.position}
                            </Badge>
                            {operator.nationId && (
                                <Badge className="border-border bg-secondary/50" variant="outline">
                                    {formatNationId(operator.nationId)}
                                </Badge>
                            )}
                        </motion.div>
                    </div>

                    {/* Right side - Avatar and faction */}
                    <motion.div animate={{ opacity: 1, x: 0 }} className="hidden items-center gap-4 md:flex" initial={{ opacity: 0, x: 20 }} transition={{ duration: 0.4, delay: 0.3 }}>
                        {(operator.nationId ?? operator.teamId ?? operator.groupId) && (
                            <div className="flex h-16 w-16 items-center justify-center rounded-lg border border-border/50 bg-card/50 p-2 backdrop-blur-sm">
                                <Image alt={operator.nationId ?? operator.teamId ?? operator.groupId ?? "Faction"} className="object-contain opacity-70" height={48} src={`/api/cdn/upk/spritepack/ui_camp_logo_0/logo_${operator.teamId ?? operator.groupId ?? operator.nationId ?? "rhodes"}.png`} width={48} />
                            </div>
                        )}
                    </motion.div>
                </div>
            </div>
        </div>
    );
}
