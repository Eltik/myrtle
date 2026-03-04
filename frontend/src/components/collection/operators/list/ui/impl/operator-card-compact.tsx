"use client";

import Image from "next/image";
import Link from "next/link";
import { memo } from "react";
import { formatSubProfession, rarityToNumber } from "~/lib/utils";
import type { OperatorFromList } from "~/types/api";
import { RARITY_COLORS } from "../../constants";
import { ClassIcon } from "./class-icon";

interface OperatorCardCompactProps {
    operator: OperatorFromList;
}

export const OperatorCardCompact = memo(function OperatorCardCompact({ operator }: OperatorCardCompactProps) {
    const rarityNum = rarityToNumber(operator.rarity);
    const rarityColor = RARITY_COLORS[rarityNum] ?? "#ffffff";
    const operatorId = operator.id ?? "";
    const operatorName = operator.name ?? "Unknown";

    const reg = /( the )|\(/gi;
    const splitName = operatorName.replace(/\)$/, "").split(reg);
    const displayName = splitName[0] ?? operatorName;
    const subtitle = splitName[2] ?? null;
    const nameIsLong = displayName.split(" ").length > 1 && displayName.length >= 16;

    return (
        <Link className="group relative flex flex-col rounded bg-card transition-transform hover:scale-102" href={`/collection/operators?id=${operatorId}`} style={{ padding: "4px 8px 4px 6px" }}>
            {/* Operator Name - Top */}
            <div className="ml-px flex h-4.25 flex-col justify-center text-left sm:h-5">
                {subtitle && <span className="text-[0.4375rem] text-foreground leading-normal sm:text-[0.5625rem] sm:leading-loose">{subtitle}</span>}
                <span
                    className="truncate text-foreground"
                    style={{
                        fontSize: nameIsLong ? "9px" : "12px",
                        lineHeight: nameIsLong ? "9px" : "17px",
                    }}
                >
                    {displayName}
                </span>
            </div>

            {/* Avatar Container with bottom rarity border */}
            <div className="relative box-content aspect-square h-20 sm:h-30" style={{ borderBottom: `4px solid ${rarityColor}` }}>
                <Image alt={operatorName} className="h-full w-full object-contain" height={120} loading="lazy" src={`/api/cdn/avatar/${operatorId}`} unoptimized width={120} />

                {/* Class icon overlay - bottom right */}
                <div className="absolute right-0 bottom-0 h-4 w-4 opacity-70 group-hover:opacity-100 sm:h-5 sm:w-5">
                    <ClassIcon profession={operator.profession} size={80} />
                </div>
            </div>

            {/* Archetype text below avatar */}
            <div className="mt-0.5 truncate text-center text-[0.5625rem] text-muted-foreground leading-tight sm:text-xs">{formatSubProfession(operator.subProfessionId.toLowerCase())}</div>
        </Link>
    );
});
