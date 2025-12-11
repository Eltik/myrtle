"use client";

import { motion } from "motion/react";
import Image from "next/image";
import { Cross, Swords, Shield, CircleGauge, Diamond, ShieldBan, Hourglass, BadgeDollarSign } from "lucide-react";
import type { Operator, AttributeData } from "~/types/api";
import { AnimatedNumber } from "~/components/ui/motion-primitives/animated-number";
import { cn } from "~/lib/utils";

interface OperatorStatsProps {
    operator: Operator;
    attributeStats: AttributeData | null;
    phaseIndex: number;
    onPhaseChange: (index: number) => void;
}

const statConfig = [
    { key: "MaxHp", label: "Health", icon: Cross },
    { key: "Atk", label: "ATK", icon: Swords },
    { key: "Def", label: "DEF", icon: Shield },
    { key: "AttackSpeed", label: "ATK Interval", icon: CircleGauge, isDecimal: true },
    { key: "MagicResistance", label: "RES", icon: Diamond },
    { key: "BlockCnt", label: "Block", icon: ShieldBan },
    { key: "RespawnTime", label: "Redeploy", icon: Hourglass },
    { key: "Cost", label: "DP Cost", icon: BadgeDollarSign },
] as const;

export function OperatorStats({ operator, attributeStats, phaseIndex, onPhaseChange }: OperatorStatsProps) {
    return (
        <div className="mt-6 space-y-4">
            {/* Elite phase tabs */}
            <div className="flex gap-2 rounded-lg bg-muted p-1">
                {operator.phases.map((_, index) => (
                    <button key={index} onClick={() => onPhaseChange(index)} className={cn("relative flex items-center justify-center rounded-md px-4 py-2 transition-all", phaseIndex === index ? "text-foreground" : "text-muted-foreground hover:text-foreground")}>
                        {phaseIndex === index && <motion.div layoutId="eliteTab" className="absolute inset-0 rounded-md bg-card shadow-sm" transition={{ type: "spring", bounce: 0.2, duration: 0.6 }} />}
                        <Image src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/elite/${index}.png`} width={32} height={32} alt={`Elite ${index}`} className="relative z-10" unoptimized />
                    </button>
                ))}
            </div>

            {/* Stats grid */}
            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                {statConfig.map((stat) => {
                    const Icon = stat.icon;
                    const value = attributeStats?.[stat.key as keyof AttributeData];
                    const displayValue = typeof value === "number" ? (stat.isDecimal ? value.toFixed(2) : Math.round(value)) : 0;

                    return (
                        <motion.div key={stat.key} className="flex items-center justify-between rounded-lg bg-muted/50 p-4" initial={{ opacity: 0, scale: 0.95 }} animate={{ opacity: 1, scale: 1 }} transition={{ duration: 0.2 }}>
                            <div className="flex items-center gap-3">
                                <Icon className="h-5 w-5 text-muted-foreground" />
                                <span className="text-sm text-muted-foreground">{stat.label}</span>
                            </div>
                            <span className="font-semibold tabular-nums">{stat.isDecimal ? displayValue : <AnimatedNumber value={Number(displayValue)} springOptions={{ bounce: 0, duration: 500 }} />}</span>
                        </motion.div>
                    );
                })}
            </div>
        </div>
    );
}
