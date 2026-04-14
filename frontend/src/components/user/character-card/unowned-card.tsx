"use client";

import Image from "next/image";
import { useState } from "react";
import { RARITY_COLORS } from "~/components/collection/operators/list/constants";
import { ImageWithSkeleton } from "~/components/ui/image-with-skeleton";
import { Card, CardContent } from "~/components/ui/shadcn/card";
import { Progress } from "~/components/ui/shadcn/progress";
import { Separator } from "~/components/ui/shadcn/separator";
import { formatProfession, getProfessionIconName, getRarityStarCount } from "~/lib/utils";
import type { UnownedOperator } from "~/types/api/impl/user";

interface UnownedCharacterCardProps {
    data: UnownedOperator;
    viewMode: "detailed" | "compact";
}

function UnownedDetailedCard({ data }: { data: UnownedOperator }) {
    const [isHovered, setIsHovered] = useState(false);
    const starCount = getRarityStarCount(data.rarity);
    const profession = formatProfession(data.profession);

    return (
        <Card
            className="fade-in slide-in-from-bottom-4 flex w-full animate-in flex-col gap-0 overflow-hidden border-2 border-muted/30 py-0 pb-1 opacity-60 grayscale transition-all duration-300 hover:border-muted hover:opacity-80 hover:shadow-lg"
            onMouseEnter={() => setIsHovered(true)}
            onMouseLeave={() => setIsHovered(false)}
        >
            {/* Image area wrapper */}
            <div className="relative">
                <div className="relative h-64 w-full overflow-hidden">
                    <ImageWithSkeleton
                        alt={data.name}
                        className={`h-full w-full object-contain object-top transition-transform duration-300 ${isHovered ? "scale-105" : "scale-100"}`}
                        containerClassName="h-full w-full"
                        height={512}
                        src={data.portrait ? `/api/cdn${data.portrait}` : `/api/cdn/avatar/${data.charId}`}
                        unoptimized
                        width={512}
                    />
                    <div className={`absolute inset-0 bg-linear-to-t from-black/50 to-transparent transition-opacity duration-300 ${isHovered ? "opacity-90" : "opacity-70"}`} />

                    {/* Operator Info Overlay */}
                    <div className="absolute right-0 bottom-0 left-0 p-4">
                        <h3 className={`mt-2 max-w-3/4 text-left font-bold text-white text-xl transition-all duration-300 ${isHovered ? "translate-y-0" : "translate-y-1"}`}>{data.name}</h3>
                        <div className={`flex items-center justify-between transition-all duration-300 ${isHovered ? "translate-y-0" : "translate-y-1"}`}>
                            <div className="flex items-center gap-2">
                                <Image alt={`${starCount} Star`} className="h-4.5 w-auto object-contain" height={18} src={`/api/cdn/upk/arts/rarity_hub/rarity_yellow_${starCount - 1}.png`} unoptimized width={60} />
                                <div className="flex flex-row items-center gap-1">
                                    <Image alt={profession} className="h-5 w-5" height={20} src={`/api/cdn/upk/arts/ui/[uc]charcommon/icon_profession_${getProfessionIconName(data.profession)}.png`} unoptimized width={20} />
                                    <span className="text-sm text-white">{profession}</span>
                                </div>
                            </div>
                            <Image alt="Elite 0" className="h-6 w-6 object-contain" height={24} src="/api/cdn/upk/arts/elite_hub/elite_0.png" unoptimized width={24} />
                        </div>
                    </div>
                </div>

                {/* Not Owned indicator (mirrors the Maxed badge position) */}
                <div className="absolute top-2 z-10 rounded-r-md bg-muted/80 px-2 py-0.5 text-center font-semibold text-muted-foreground text-xs shadow-md">Not Owned</div>
            </div>

            {/* Operator Stats - placeholder to match owned card structure */}
            <CardContent className="min-w-0 flex-1 overflow-hidden px-4 pt-2 pb-2">
                <div className="grid grid-cols-2 gap-4">
                    <div>
                        <div className="flex items-center justify-between">
                            <span className="font-medium text-sm">Level</span>
                            <span className="font-bold text-muted-foreground text-sm">--</span>
                        </div>
                        <Progress className="h-1.5" value={0} />
                    </div>
                    <div>
                        <div className="flex items-center justify-between">
                            <span className="font-medium text-sm">Trust</span>
                            <span className="font-bold text-muted-foreground text-sm">--</span>
                        </div>
                        <Progress className="h-1.5" value={0} />
                    </div>
                </div>

                <Separator className="my-3" />

                <div className="grid grid-cols-2 gap-x-4 gap-y-1 text-sm">
                    {["HP", "ATK", "DEF", "RES", "Cost", "Block"].map((stat) => (
                        <div className="flex items-center justify-between" key={stat}>
                            <span className="text-muted-foreground">{stat}</span>
                            <span className="font-medium text-muted-foreground">--</span>
                        </div>
                    ))}
                </div>

                <Separator className="my-3" />

                {/* Placeholder accordion rows to match owned card height */}
                <div className="w-full">
                    {["Potential", "Skills", "Modules"].map((label) => (
                        <div className="flex items-center justify-between border-b-0 py-2" key={label}>
                            <span className="font-medium text-muted-foreground/50 text-sm">{label}</span>
                        </div>
                    ))}
                </div>
            </CardContent>
        </Card>
    );
}

function UnownedCompactCard({ data }: { data: UnownedOperator }) {
    const starCount = getRarityStarCount(data.rarity);
    const rarityColor = RARITY_COLORS[starCount] ?? "#ffffff";

    const reg = /( the )|\(/gi;
    const splitName = data.name.replace(/\)$/, "").split(reg);
    const displayName = splitName[0] ?? data.name;
    const subtitle = splitName[2] ?? null;
    const nameIsLong = displayName.split(" ").length > 1 && displayName.length >= 16;

    return (
        <div>
            <div
                className="fade-in slide-in-from-bottom-2 relative flex h-min w-min animate-in flex-col rounded bg-card opacity-60 grayscale"
                style={{
                    padding: "4px 8px 4px 6px",
                    margin: "2px 4px 4px 10px",
                    boxShadow: "0 1px 3px 0 rgb(0 0 0 / 0.1)",
                }}
            >
                {/* Operator Name - Top */}
                <div className="ml-px flex h-4.25 flex-col justify-center text-left sm:h-5">
                    {subtitle && <span className="text-[0.4375rem] text-foreground leading-normal sm:text-[0.5625rem] sm:leading-loose">{subtitle}</span>}
                    <span
                        className="text-foreground"
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
                    <ImageWithSkeleton alt={data.name} className="h-full w-full object-contain" containerClassName="h-full w-full" height={120} src={`/api/cdn/avatar/${data.charId}`} unoptimized width={120} />
                </div>
            </div>
        </div>
    );
}

export function UnownedCharacterCard({ data, viewMode }: UnownedCharacterCardProps) {
    if (viewMode === "detailed") {
        return <UnownedDetailedCard data={data} />;
    }
    return <UnownedCompactCard data={data} />;
}
