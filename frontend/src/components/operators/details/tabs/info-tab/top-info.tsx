"use client";

import Image from "next/image";
import { Star } from "lucide-react";
import { motion } from "motion/react";
import type { Operator } from "~/types/api";
import { rarityToNumber, formatNationId, formatGroupId, formatProfession, formatSubProfession, getAvatarById, getProfessionImage, getSubProfessionImage } from "~/lib/operator-helpers";

interface TopInfoProps {
    operator: Operator;
}

export function TopInfo({ operator }: TopInfoProps) {
    const rarity = rarityToNumber(operator.rarity);

    return (
        <div className="flex w-full flex-col gap-4 lg:flex-row lg:items-start">
            {/* Avatar and Name Section */}
            <div className="flex items-start gap-4">
                {/* Avatar */}
                <motion.div whileHover={{ scale: 1.05 }} className="relative flex-shrink-0">
                    <div className="relative h-24 w-24 overflow-hidden rounded-lg border-2 border-border bg-muted/50 md:h-28 md:w-28">
                        <Image src={getAvatarById(operator.id ?? "")} alt={operator.name} fill className="object-cover" />
                    </div>
                    {/* Rarity Stars */}
                    <div className="absolute -bottom-2 left-1/2 flex -translate-x-1/2 gap-0.5">
                        {Array.from({ length: rarity }).map((_, i) => (
                            <Star key={i} size={14} className="fill-primary stroke-background" strokeWidth={2} />
                        ))}
                    </div>
                </motion.div>

                {/* Name and Class */}
                <div className="flex flex-col gap-2">
                    <h3 className="text-2xl font-bold md:text-3xl">{operator.name}</h3>
                    <div className="flex items-center gap-2 rounded-md border border-border overflow-hidden">
                        <div className="flex items-center justify-center bg-card p-2">
                            <Image src={getProfessionImage(operator.profession) || "/placeholder.svg"} alt={formatProfession(operator.profession)} width={32} height={32} className="h-8 w-8" />
                        </div>
                        <div className="flex items-center gap-2 bg-muted px-3 py-2">
                            <Image src={getSubProfessionImage(operator.subProfessionId) || "/placeholder.svg"} alt={formatSubProfession(operator.subProfessionId)} width={24} height={24} className="h-6 w-6" />
                            <span className="text-sm font-medium">{formatSubProfession(operator.subProfessionId)}</span>
                        </div>
                    </div>
                </div>
            </div>

            {/* Info Grid */}
            <div className="mt-4 grid flex-1 grid-cols-3 gap-2 lg:mt-0 lg:justify-end">
                <InfoItem label="Nation" value={operator.nationId ? formatNationId(operator.nationId) : "N/A"} />
                <InfoItem label="Faction" value={operator.groupId ? formatGroupId(operator.groupId) : "N/A"} />
                <InfoItem label="Position" value={operator.position === "MELEE" ? "Melee" : "Ranged"} />
            </div>
        </div>
    );
}

function InfoItem({ label, value }: { label: string; value: string }) {
    return (
        <div className="flex flex-col rounded-md bg-muted/50 p-3">
            <span className="text-xs text-muted-foreground">{label}</span>
            <span className="truncate text-sm font-medium">{value}</span>
        </div>
    );
}
