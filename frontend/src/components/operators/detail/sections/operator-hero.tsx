"use client";

import { ChevronRight } from "lucide-react";
import { motion, useScroll, useTransform } from "motion/react";
import Image from "next/image";
import Link from "next/link";
import { useRef } from "react";
import { Badge } from "~/components/ui/shadcn/badge";
import { cn, formatNationId, formatProfession, formatSubProfession, rarityToNumber } from "~/lib/utils";
import type { Operator } from "~/types/api";

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

const RARITY_GLOW: Record<string, string> = {
    TIER_6: "drop-shadow-[0_0_20px_rgba(255,154,74,0.5)]",
    TIER_5: "drop-shadow-[0_0_20px_rgba(255,230,109,0.4)]",
    TIER_4: "drop-shadow-[0_0_15px_rgba(201,184,240,0.4)]",
    TIER_3: "drop-shadow-[0_0_15px_rgba(125,211,252,0.4)]",
    TIER_2: "drop-shadow-[0_0_15px_rgba(134,239,172,0.3)]",
    TIER_1: "drop-shadow-[0_0_10px_rgba(255,255,255,0.2)]",
};

export function OperatorHero({ operator }: OperatorHeroProps) {
    const containerRef = useRef<HTMLDivElement>(null);

    const rarityNum = rarityToNumber(operator.rarity);
    const rarityColor = RARITY_COLORS[operator.rarity] ?? RARITY_COLORS.TIER_1;
    const rarityGlow = RARITY_GLOW[operator.rarity] ?? RARITY_GLOW.TIER_1;

    const { scrollYProgress } = useScroll({
        target: containerRef,
        offset: ["start start", "end start"],
    });

    const imageY = useTransform(scrollYProgress, [0, 1], [0, 100]);
    const imageScale = useTransform(scrollYProgress, [0, 1], [1, 1.08]);
    const contentOpacity = useTransform(scrollYProgress, [0, 0.4], [1, 0]);
    const contentY = useTransform(scrollYProgress, [0, 0.4], [0, -40]);

    const operatorId = operator.id ?? "";
    const heroImageUrl = operator.skin ? `/api/cdn${operator.skin}` : `/api/cdn/upk/chararts/${operatorId}/${operatorId}_2.png`;

    return (
        <div className="relative h-[280px] w-full overflow-hidden contain-layout sm:h-80 md:h-[380px] lg:h-[420px]" ref={containerRef}>
            {/* Background Image */}
            <motion.div
                className="backface-hidden absolute inset-x-0 top-0 transition-transform duration-75 ease-out will-change-transform contain-paint"
                style={{
                    y: imageY,
                    scale: imageScale,
                }}
            >
                <div className="flex items-start justify-center pt-0 md:justify-end md:pr-[5%] lg:pr-[10%]">
                    <div className="relative h-[480px] w-[85vw] max-w-[380px] sm:h-[540px] sm:w-[440px] sm:max-w-none md:h-[620px] md:w-[520px] lg:h-[720px] lg:w-[600px]">
                        <Image alt={operator.name} className={cn("object-contain object-top", rarityGlow)} fill priority sizes="(max-width: 640px) 85vw, (max-width: 768px) 440px, (max-width: 1024px) 520px, 600px" src={heroImageUrl} />
                    </div>
                </div>
            </motion.div>

            <div className="-bottom-16 sm:-bottom-14 pointer-events-none absolute inset-x-0 h-[calc(33%+64px)] bg-linear-to-t from-background via-background/80 to-transparent sm:h-[calc(33%+56px)] lg:bottom-0 lg:h-1/3" />

            {/* Content */}
            <motion.div
                className="backface-hidden relative z-10 mx-auto flex h-full max-w-6xl flex-col justify-end px-3 pb-4 transition-[transform,opacity] duration-75 ease-out will-change-[transform,opacity] sm:px-4 sm:pb-5 md:px-8 md:pb-6 lg:pb-8"
                style={{
                    opacity: contentOpacity,
                    y: contentY,
                }}
            >
                {/* Breadcrumb */}
                <motion.nav animate={{ opacity: 1, y: 0 }} className="mb-2 md:mb-3" initial={{ opacity: 0, y: -10 }} transition={{ duration: 0.3 }}>
                    <ol className="flex items-center gap-2 text-muted-foreground text-sm">
                        <li>
                            <Link className="transition-colors hover:text-foreground" href="/operators/list">
                                Operators
                            </Link>
                        </li>
                        <ChevronRight className="h-4 w-4" />
                        <li className="font-medium text-foreground">{operator.name}</li>
                    </ol>
                </motion.nav>

                {/* Operator Info */}
                <div className="flex flex-col gap-2 sm:gap-3 md:flex-row md:items-end md:justify-between md:gap-4">
                    <div className="flex flex-col gap-1.5 sm:gap-2">
                        {/* Name and Rarity */}
                        <motion.div animate={{ opacity: 1, y: 0 }} initial={{ opacity: 0, y: 20 }} transition={{ duration: 0.4, delay: 0.1 }}>
                            <h1 className="font-bold text-2xl text-foreground tracking-tight sm:text-3xl md:text-4xl lg:text-5xl">{operator.name}</h1>
                            <div className="mt-1 flex items-center gap-2 sm:mt-1.5 sm:gap-3">
                                <span className={cn("font-semibold text-lg tracking-wider", rarityColor)}>{Array(rarityNum).fill("★").join("")}</span>
                                <span className="text-muted-foreground/50">|</span>
                                <span className="text-muted-foreground text-sm md:text-base">{formatSubProfession(operator.subProfessionId)}</span>
                            </div>
                        </motion.div>

                        {/* Tags */}
                        <motion.div animate={{ opacity: 1, y: 0 }} className="flex flex-wrap gap-2" initial={{ opacity: 0, y: 20 }} transition={{ duration: 0.4, delay: 0.2 }}>
                            <Badge className="border-transparent bg-accent text-foreground" variant="outline">
                                {formatProfession(operator.profession)}
                            </Badge>
                            <Badge className="border-transparent bg-accent text-foreground" variant="outline">
                                {operator.position === "RANGED" ? "Ranged" : operator.position === "MELEE" ? "Melee" : operator.position}
                            </Badge>
                            {operator.nationId && (
                                <Badge className="border-transparent bg-accent text-foreground" variant="outline">
                                    {formatNationId(operator.nationId)}
                                </Badge>
                            )}
                        </motion.div>
                    </div>

                    {/* Right side - Avatar and faction */}
                    <motion.div animate={{ opacity: 1, x: 0 }} className="hidden items-center gap-4 md:flex" initial={{ opacity: 0, x: 20 }} transition={{ duration: 0.4, delay: 0.3 }}>
                        {((operator.nationId && operator.nationId.length > 0) ?? (operator.teamId && operator.teamId.length > 0) ?? (operator.groupId && operator.groupId.length > 0)) && (
                            <div className="flex h-14 w-14 items-center justify-center rounded-full bg-black/30 p-2.5 lg:h-16 lg:w-16">
                                <Image
                                    alt={operator.nationId && operator.nationId.length > 0 ? operator.nationId : operator.teamId && operator.teamId.length > 0 ? operator.teamId : operator.groupId && operator.groupId.length > 0 ? operator.groupId : "Faction"}
                                    className="object-contain opacity-80"
                                    height={44}
                                    src={`/api/cdn/upk/spritepack/ui_camp_logo_0/logo_${operator.nationId && operator.nationId.length > 0 ? operator.nationId : (operator.teamId && operator.teamId.length > 0) ? operator.teamId : operator.groupId && operator.groupId.length > 0 ? operator.groupId : "rhodes"}.png`}
                                    width={48}
                                />
                            </div>
                        )}
                    </motion.div>
                </div>
            </motion.div>
        </div>
    );
}
