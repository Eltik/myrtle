"use client";

import Image from "next/image";
import Link from "next/link";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "~/components/ui/hover-card";
import { capitalize, cn, formatProfession, formatSubProfession, rarityToNumber } from "~/lib/utils";
import type { OperatorFromList } from "~/types/api/operators";

interface OperatorCardProps {
    operator: OperatorFromList;
    viewMode: "grid" | "list";
    isHovered?: boolean;
    shouldGrayscale?: boolean;
    onHoverChange?: (isOpen: boolean) => void;
}

const RarityColors: Record<number, string> = {
    6: "#f7a452",
    5: "#f7e79e",
    4: "#bcabdb",
    3: "#88c8e3",
    2: "#7ef2a3",
    1: "#ffffff",
};

const RarityBlurColors: Record<number, string> = {
    6: "#cc9b6a",
    5: "#d6c474",
    4: "#9e87c7",
    3: "#62a2bd",
    2: "#57ab72",
    1: "#aaaaaa",
};

const HOVER_DELAY = 500;

export function OperatorCard({ operator, viewMode, isHovered = false, shouldGrayscale = false, onHoverChange }: OperatorCardProps) {
    const rarityNum = rarityToNumber(operator.rarity);
    const rarityColor = RarityColors[rarityNum] ?? "#ffffff";
    const rarityBlurColor = RarityBlurColors[rarityNum] ?? "#aaaaaa";
    const operatorId = operator.id!;

    if (viewMode === "list") {
        return (
            <Link className={cn("group flex items-center gap-4 rounded-lg border border-muted/50 bg-card p-3 transition-all duration-200 hover:bg-card/80", shouldGrayscale && "grayscale", isHovered && "grayscale-0")} href={`/operators/${operatorId}`}>
                <div className="relative h-12 w-12 shrink-0 overflow-hidden rounded-lg">
                    <Image alt={operator.name} className="object-cover" fill src={`/api/cdn${operator.portrait}`} />
                </div>
                <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                        <span className="truncate font-semibold text-foreground">{operator.name}</span>
                        <span className="text-amber-400 text-sm">{"★".repeat(rarityNum)}</span>
                    </div>
                    <div className="flex items-center gap-2 text-muted-foreground text-sm">
                        <span>{formatProfession(operator.profession)}</span>
                        <span className="text-border">•</span>
                        <span>{capitalize(formatSubProfession(operator.subProfessionId.toLowerCase()))}</span>
                    </div>
                </div>
            </Link>
        );
    }

    const cardContent = (
        <Link aria-label={`View details for ${operator.name}`} className="group relative flex aspect-2/3 overflow-clip rounded-md border border-muted/50 bg-card transition hover:rounded-lg" href={`/operators/${operatorId}`}>
            {/* Faction background */}
            <div className="-translate-x-8 -translate-y-4 absolute">
                <Image
                    alt={String(operator.nationId ?? operator.teamId ?? "Rhodes Island")}
                    className="opacity-5 transition-opacity group-hover:opacity-10"
                    decoding="async"
                    height={360}
                    loading="lazy"
                    src={operator.nationId ? `/api/cdn/upk/spritepack/ui_camp_logo_0/logo_${String(operator.nationId)}.png` : operator.teamId ? `/api/cdn/upk/spritepack/ui_camp_logo_0/logo_${operator.teamId}.png` : `/api/cdn/upk/spritepack/ui_camp_logo_0/logo_rhodes.png`}
                    width={360}
                />
            </div>

            {/* Portrait */}
            <div className="absolute inset-0">
                <div className={cn("relative h-full w-full scale-100 transition-all duration-150 group-hover:scale-105", shouldGrayscale && "grayscale", isHovered && "grayscale-0")}>
                    <Image alt={`${operator.name} Portrait`} className="h-full w-full rounded-lg object-contain" decoding="async" fill loading="lazy" src={`/api/cdn${operator.portrait}`} />
                </div>
            </div>

            {/* Bottom info bar */}
            <div className="absolute inset-x-0 bottom-0 z-10">
                <div className="relative">
                    <div className="h-12 w-full bg-background/80 backdrop-blur-sm" />
                    <h2 className="absolute bottom-1 left-1 line-clamp-2 max-w-[85%] pr-8 font-bold text-xs uppercase opacity-60 transition-opacity group-hover:opacity-100 sm:text-sm md:text-sm">{operator.name}</h2>
                    {/* Class icon */}
                    <div className="absolute right-1 bottom-1 flex scale-75 items-center opacity-0 transition-all duration-200 group-hover:scale-100 group-hover:opacity-100">
                        <div className="h-4 w-4 md:h-6 md:w-6">
                            <Image alt={formatProfession(operator.profession)} decoding="async" height={160} loading="lazy" src={`/api/cdn/upk/arts/ui/[uc]charcommon/icon_profession_${operator.profession.toLowerCase()}.png`} width={160} />
                        </div>
                    </div>
                    {/* Rarity color bar */}
                    <div className={cn("absolute bottom-0 h-0.5 w-full", shouldGrayscale && "grayscale", isHovered && "grayscale-0")} style={{ backgroundColor: rarityColor }} />
                    <div className={cn("-bottom-0.5 absolute h-1 w-full blur-sm", shouldGrayscale && "grayscale", isHovered && "grayscale-0")} style={{ backgroundColor: rarityBlurColor }} />
                </div>
            </div>
        </Link>
    );

    return (
        <HoverCard closeDelay={50} onOpenChange={onHoverChange} openDelay={HOVER_DELAY}>
            <HoverCardTrigger asChild>{cardContent}</HoverCardTrigger>
            <HoverCardContent className="w-80 p-4" side="top">
                <div className="flex items-start space-x-4">
                    {/* Avatar */}
                    <div className="relative h-16 w-16 shrink-0">
                        <Image alt={`${operator.name} Avatar`} className="rounded-md object-cover" fill src={`/api/cdn${operator.portrait}`} />
                    </div>
                    <div className="grow space-y-1">
                        {/* Name and faction */}
                        <div className="flex items-center justify-between">
                            <h4 className="font-semibold text-base">{operator.name}</h4>
                            <div className="h-6 w-6 shrink-0">
                                <Image
                                    alt={String(operator.nationId ?? operator.teamId ?? "Rhodes Island")}
                                    className="object-contain"
                                    height={24}
                                    src={operator.nationId ? `/api/cdn/upk/spritepack/ui_camp_logo_0/logo_${String(operator.nationId)}.png` : operator.teamId ? `/api/cdn/upk/spritepack/ui_camp_logo_0/logo_${operator.teamId}.png` : `/api/cdn/upk/spritepack/ui_camp_logo_0/logo_rhodes.png`}
                                    width={24}
                                />
                            </div>
                        </div>
                        {/* Rarity and class info */}
                        <p className="font-semibold text-xs" style={{ color: rarityColor }}>
                            {`${rarityNum}★ ${capitalize(formatSubProfession(operator.subProfessionId.toLowerCase()))} ${formatProfession(operator.profession)}`}
                        </p>
                        {/* Position and race */}
                        <div className="flex space-x-2 pt-1 text-muted-foreground text-xs">
                            <span>{capitalize(operator.position?.toLowerCase() ?? "Unknown")}</span>
                            {operator.profile?.basicInfo?.race && (
                                <>
                                    <span>•</span>
                                    <span>{operator.profile.basicInfo.race}</span>
                                </>
                            )}
                        </div>
                    </div>
                </div>
            </HoverCardContent>
        </HoverCard>
    );
}
