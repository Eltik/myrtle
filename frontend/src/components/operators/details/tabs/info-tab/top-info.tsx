"use client";

import { Star } from "lucide-react";
import { motion } from "motion/react";
import Image from "next/image";
import { formatGroupId, formatNationId, formatProfession, formatSubProfession, getAvatarById, getProfessionImage, getSubProfessionImage, rarityToNumber } from "~/lib/operator-helpers";
import type { Operator } from "~/types/api";

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
                <motion.div className="relative flex-shrink-0" whileHover={{ scale: 1.05 }}>
                    <div className="relative h-24 w-24 overflow-hidden rounded-lg border-2 border-border bg-muted/50 md:h-28 md:w-28">
                        <Image alt={operator.name} className="object-cover" fill src={getAvatarById(operator.id ?? "")} />
                    </div>
                    {/* Rarity Stars */}
                    <div className="-bottom-2 -translate-x-1/2 absolute left-1/2 flex gap-0.5">
                        {Array.from({ length: rarity }).map((_, i) => (
                            <Star className="fill-primary stroke-background" key={i} size={14} strokeWidth={2} />
                        ))}
                    </div>
                </motion.div>

                {/* Name and Class */}
                <div className="flex flex-col gap-2">
                    <h3 className="font-bold text-2xl md:text-3xl">{operator.name}</h3>
                    <div className="flex items-center gap-2 overflow-hidden rounded-md border border-border">
                        <div className="flex items-center justify-center bg-card p-2">
                            <Image alt={formatProfession(operator.profession)} className="h-8 w-8" height={32} src={getProfessionImage(operator.profession) || "/placeholder.svg"} width={32} />
                        </div>
                        <div className="flex items-center gap-2 bg-muted px-3 py-2">
                            <Image alt={formatSubProfession(operator.subProfessionId)} className="h-6 w-6" height={24} src={getSubProfessionImage(operator.subProfessionId) || "/placeholder.svg"} width={24} />
                            <span className="font-medium text-sm">{formatSubProfession(operator.subProfessionId)}</span>
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
            <span className="text-muted-foreground text-xs">{label}</span>
            <span className="truncate font-medium text-sm">{value}</span>
        </div>
    );
}
