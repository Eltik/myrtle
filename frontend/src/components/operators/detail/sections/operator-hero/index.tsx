"use client";

import { ChevronRight } from "lucide-react";
import { motion, useScroll, useTransform } from "motion/react";
import Image from "next/image";
import Link from "next/link";
import { useRef } from "react";
import { Badge } from "~/components/ui/shadcn/badge";
import { cn, formatNationId, formatProfession, formatSubProfession, rarityToNumber } from "~/lib/utils";
import type { Operator } from "~/types/api";
import { RARITY_COLORS, RARITY_GLOW } from "./impl/constants";

interface OperatorHeroProps {
    operator: Operator;
}

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
        <div className="relative w-full overflow-hidden contain-layout" ref={containerRef}>
            {/* Mobile Layout: Similar to desktop with gradient fade */}
            <div className="md:hidden">
                <div className="relative h-[360px] w-full overflow-hidden sm:h-[400px]">
                    {/* Background Image with parallax */}
                    <motion.div
                        className="backface-hidden absolute inset-x-0 top-0 transition-transform duration-75 ease-out will-change-transform contain-paint"
                        style={{
                            y: imageY,
                            scale: imageScale,
                        }}
                    >
                        <div className="flex items-start justify-center pt-0">
                            <div className="relative h-[480px] w-[85vw] max-w-[380px] sm:h-[540px] sm:w-[440px] sm:max-w-none">
                                <Image alt={operator.name} className={cn("object-contain object-top", rarityGlow)} fill priority sizes="(max-width: 640px) 85vw, 440px" src={heroImageUrl} />
                            </div>
                        </div>
                    </motion.div>

                    {/* Gradient fade - similar to desktop */}
                    <div className="pointer-events-none absolute inset-x-0 bottom-0 h-1/2 bg-linear-to-t from-background via-background/80 to-transparent" />

                    {/* Content - positioned at bottom like desktop */}
                    <motion.div
                        className="backface-hidden relative z-10 flex h-full flex-col justify-end px-3 pb-4 transition-[transform,opacity] duration-75 ease-out will-change-[transform,opacity] sm:px-4 sm:pb-5"
                        style={{
                            opacity: contentOpacity,
                            y: contentY,
                        }}
                    >
                        {/* Breadcrumb */}
                        <motion.nav animate={{ opacity: 1, y: 0 }} className="mb-2" initial={{ opacity: 0, y: -10 }} transition={{ duration: 0.3 }}>
                            <ol className="flex items-center gap-2 text-muted-foreground text-xs">
                                <li>
                                    <Link className="transition-colors hover:text-foreground" href="/operators/list">
                                        Operators
                                    </Link>
                                </li>
                                <ChevronRight className="h-3 w-3" />
                                <li className="font-medium text-foreground">{operator.name}</li>
                            </ol>
                        </motion.nav>

                        {/* Name and Rarity */}
                        <motion.div animate={{ opacity: 1, y: 0 }} initial={{ opacity: 0, y: 20 }} transition={{ duration: 0.4, delay: 0.1 }}>
                            <h1 className="font-bold text-2xl text-foreground tracking-tight sm:text-3xl">{operator.name}</h1>
                            <div className="mt-1.5 flex items-center gap-2">
                                <span className={cn("font-semibold text-base tracking-wider", rarityColor)}>{Array(rarityNum).fill("★").join("")}</span>
                                <span className="text-muted-foreground/50">|</span>
                                <span className="text-muted-foreground text-sm">{formatSubProfession(operator.subProfessionId)}</span>
                            </div>
                        </motion.div>

                        {/* Tags */}
                        <motion.div animate={{ opacity: 1, y: 0 }} className="mt-3 flex flex-wrap gap-2" initial={{ opacity: 0, y: 20 }} transition={{ duration: 0.4, delay: 0.2 }}>
                            <Badge className="border-transparent bg-accent text-foreground text-xs" variant="outline">
                                {formatProfession(operator.profession)}
                            </Badge>
                            <Badge className="border-transparent bg-accent text-foreground text-xs" variant="outline">
                                {operator.position === "RANGED" ? "Ranged" : operator.position === "MELEE" ? "Melee" : operator.position}
                            </Badge>
                            {operator.nationId && (
                                <Badge className="border-transparent bg-accent text-foreground text-xs" variant="outline">
                                    {formatNationId(operator.nationId)}
                                </Badge>
                            )}
                        </motion.div>
                    </motion.div>
                </div>
            </div>

            {/* Desktop Layout: Original design */}
            <div className="hidden md:block">
                <div className="relative h-[380px] w-full overflow-hidden lg:h-[420px]">
                    {/* Background Image */}
                    <motion.div
                        className="backface-hidden absolute inset-x-0 top-0 transition-transform duration-75 ease-out will-change-transform contain-paint"
                        style={{
                            y: imageY,
                            scale: imageScale,
                        }}
                    >
                        <div className="flex items-start justify-end pr-[5%] lg:pr-[10%]">
                            <div className="relative h-[620px] w-[520px] lg:h-[720px] lg:w-[600px]">
                                <Image alt={operator.name} className={cn("object-contain object-top", rarityGlow)} fill priority sizes="(max-width: 1024px) 520px, 600px" src={heroImageUrl} />
                            </div>
                        </div>
                    </motion.div>

                    <div className="pointer-events-none absolute inset-x-0 bottom-0 h-1/3 bg-linear-to-t from-background via-background/80 to-transparent" />

                    {/* Content */}
                    <motion.div
                        className="backface-hidden relative z-10 mx-auto flex h-full max-w-6xl flex-col justify-end px-8 pb-6 transition-[transform,opacity] duration-75 ease-out will-change-[transform,opacity] lg:pb-8"
                        style={{
                            opacity: contentOpacity,
                            y: contentY,
                        }}
                    >
                        {/* Breadcrumb */}
                        <motion.nav animate={{ opacity: 1, y: 0 }} className="mb-3" initial={{ opacity: 0, y: -10 }} transition={{ duration: 0.3 }}>
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
                        <div className="flex flex-row items-end justify-between gap-4">
                            <div className="flex flex-col gap-2">
                                {/* Name and Rarity */}
                                <motion.div animate={{ opacity: 1, y: 0 }} initial={{ opacity: 0, y: 20 }} transition={{ duration: 0.4, delay: 0.1 }}>
                                    <h1 className="font-bold text-4xl text-foreground tracking-tight lg:text-5xl">{operator.name}</h1>
                                    <div className="mt-1.5 flex items-center gap-3">
                                        <span className={cn("font-semibold text-lg tracking-wider", rarityColor)}>{Array(rarityNum).fill("★").join("")}</span>
                                        <span className="text-muted-foreground/50">|</span>
                                        <span className="text-base text-muted-foreground">{formatSubProfession(operator.subProfessionId)}</span>
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
                            <motion.div animate={{ opacity: 1, x: 0 }} className="flex items-center gap-4" initial={{ opacity: 0, x: 20 }} transition={{ duration: 0.4, delay: 0.3 }}>
                                {((operator.nationId && operator.nationId.length > 0) ?? (operator.teamId && operator.teamId.length > 0) ?? (operator.groupId && operator.groupId.length > 0)) && (
                                    <div className="h-14 w-14 lg:h-16 lg:w-16">
                                        <Image
                                            alt={operator.nationId && operator.nationId.length > 0 ? operator.nationId : operator.teamId && operator.teamId.length > 0 ? operator.teamId : operator.groupId && operator.groupId.length > 0 ? operator.groupId : "Faction"}
                                            className="icon-theme-aware object-contain opacity-80"
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
            </div>
        </div>
    );
}
