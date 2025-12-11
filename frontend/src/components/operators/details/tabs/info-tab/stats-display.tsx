"use client";

import { BadgeDollarSign, CircleGauge, Cross, Diamond, Hourglass, Shield, ShieldBan, Swords } from "lucide-react";
import { motion } from "motion/react";
import Image from "next/image";
import { AnimatedNumber } from "~/components/ui/motion-primitives/animated-number";
import { getEliteImage } from "~/lib/operator-helpers";
import type { Operator } from "~/types/api";

interface StatsDisplayProps {
    operator: Operator;
    attributeStats: {
        maxHp: number;
        atk: number;
        def: number;
        magicResistance: number;
        attackSpeed: number;
        blockCnt: number;
        respawnTime: number;
        cost: number;
    } | null;
    phaseIndex: number;
    onPhaseChange: (index: number) => void;
}

const statConfig = [
    { key: "maxHp", label: "Health", icon: Cross },
    { key: "atk", label: "ATK", icon: Swords },
    { key: "def", label: "DEF", icon: Shield },
    { key: "attackSpeed", label: "ATK Interval", icon: CircleGauge, decimal: true },
    { key: "magicResistance", label: "RES", icon: Diamond },
    { key: "blockCnt", label: "Block", icon: ShieldBan },
    { key: "respawnTime", label: "Redeploy", icon: Hourglass },
    { key: "cost", label: "DP Cost", icon: BadgeDollarSign },
] as const;

export function StatsDisplay({ operator, attributeStats, phaseIndex, onPhaseChange }: StatsDisplayProps) {
    return (
        <div className="space-y-4">
            {/* Phase Selector */}
            <div className="flex items-center gap-2 rounded-lg bg-muted/50 p-2">
                {operator.phases.map((_, index) => (
                    <motion.button className={`relative rounded-md p-2 transition-colors ${phaseIndex === index ? "bg-card shadow-md" : "hover:bg-card/50"}`} key={index} onClick={() => onPhaseChange(index)} whileHover={{ scale: 1.05 }} whileTap={{ scale: 0.95 }}>
                        <Image alt={`Elite ${index}`} className="h-8 w-8 object-contain" height={35} src={getEliteImage(index) || "/placeholder.svg"} width={35} />
                        {phaseIndex === index && <motion.div className="absolute inset-0 rounded-md border-2 border-primary" layoutId="phase-indicator" transition={{ type: "spring", bounce: 0.2, duration: 0.4 }} />}
                    </motion.button>
                ))}
            </div>

            {/* Stats Grid */}
            <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
                {statConfig.map(({ key, label, icon: Icon, decimal }) => {
                    const value = attributeStats?.[key] ?? 0;
                    return (
                        <motion.div animate={{ opacity: 1, y: 0 }} className="flex items-center justify-between rounded-lg bg-muted/50 p-3" initial={{ opacity: 0, y: 10 }} key={key}>
                            <div className="flex items-center gap-2">
                                <Icon className="h-5 w-5 text-muted-foreground" />
                                <span className="text-muted-foreground text-sm">{label}</span>
                            </div>
                            <span className="font-bold tabular-nums">{decimal ? value.toFixed(2) : <AnimatedNumber springOptions={{ bounce: 0, duration: 500 }} value={Math.round(value)} />}</span>
                        </motion.div>
                    );
                })}
            </div>
        </div>
    );
}
