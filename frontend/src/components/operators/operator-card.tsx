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
    const operatorId = operator.id ?? "";

    if (viewMode === "list") {
        return (
            <Link className={cn("group relative flex items-center gap-3 rounded-lg border border-transparent bg-card/50 px-3 py-2.5 transition-all duration-200 hover:border-border hover:bg-card", shouldGrayscale && "grayscale", isHovered && "grayscale-0")} href={`/operators/${operatorId}`}>
                {/* Rarity indicator line on left */}
                <div className="-translate-y-1/2 absolute top-1/2 left-0 h-8 w-0.5 rounded-full opacity-60 transition-opacity group-hover:opacity-100" style={{ backgroundColor: rarityColor }} />

                {/* Portrait - fixed width for alignment */}
                <div className="relative h-12 w-12 shrink-0 overflow-hidden rounded-md border border-border/50 bg-background">
                    <Image alt={operator.name} className="object-cover transition-transform duration-200 group-hover:scale-110" fill src={`/api/cdn${operator.portrait}`} />
                </div>

                {/* Desktop Layout: Name column */}
                <div className="hidden min-w-0 flex-1 md:block">
                    <span className="truncate font-semibold text-foreground text-sm uppercase tracking-wide">{operator.name}</span>
                </div>

                {/* Desktop: Rarity stars - w-24 to match header */}
                <div className="hidden w-24 shrink-0 items-center gap-0.5 md:flex">
                    {Array.from({ length: rarityNum }).map((_, i) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: Static array of stars based on rarity count, order never changes
                        <span className="text-sm" key={i} style={{ color: rarityColor }}>
                            ★
                        </span>
                    ))}
                </div>

                {/* Desktop: Class info - w-32 to match header */}
                <div className="hidden w-32 shrink-0 items-center gap-2 md:flex">
                    <div className="flex h-5 w-5 items-center justify-center">
                        <Image alt={formatProfession(operator.profession)} className="opacity-60 transition-opacity group-hover:opacity-100" height={20} src={`/api/cdn/upk/arts/ui/[uc]charcommon/icon_profession_${operator.profession.toLowerCase()}.png`} width={20} />
                    </div>
                    <span className="text-muted-foreground text-sm">{formatProfession(operator.profession)}</span>
                </div>

                {/* Desktop: Archetype - w-40 to match header */}
                <div className="hidden w-40 shrink-0 lg:block">
                    <span className="truncate text-muted-foreground text-sm">{capitalize(formatSubProfession(operator.subProfessionId.toLowerCase()))}</span>
                </div>

                {/* Desktop: Faction logo - w-8 to match header */}
                <div className="hidden w-8 shrink-0 justify-center xl:flex">
                    <div className="flex h-6 w-6 items-center justify-center opacity-40 transition-opacity group-hover:opacity-70">
                        <Image
                            alt={String(operator.nationId ?? operator.teamId ?? "Rhodes Island")}
                            className="object-contain"
                            height={24}
                            src={operator.nationId ? `/api/cdn/upk/spritepack/ui_camp_logo_0/logo_${String(operator.nationId)}.png` : operator.teamId ? `/api/cdn/upk/spritepack/ui_camp_logo_0/logo_${operator.teamId}.png` : `/api/cdn/upk/spritepack/ui_camp_logo_0/logo_rhodes.png`}
                            width={24}
                        />
                    </div>
                </div>

                {/* Mobile Layout: Comprehensive info display */}
                <div className="flex min-w-0 flex-1 flex-col gap-1 md:hidden">
                    {/* Name row with faction icon */}
                    <div className="flex items-center gap-2">
                        <span className="truncate font-semibold text-foreground text-sm uppercase tracking-wide">{operator.name}</span>
                        <div className="flex h-4 w-4 shrink-0 items-center justify-center opacity-50">
                            <Image
                                alt={String(operator.nationId ?? operator.teamId ?? "Rhodes Island")}
                                className="object-contain"
                                height={16}
                                src={operator.nationId ? `/api/cdn/upk/spritepack/ui_camp_logo_0/logo_${String(operator.nationId)}.png` : operator.teamId ? `/api/cdn/upk/spritepack/ui_camp_logo_0/logo_${operator.teamId}.png` : `/api/cdn/upk/spritepack/ui_camp_logo_0/logo_rhodes.png`}
                                width={16}
                            />
                        </div>
                    </div>

                    {/* Info row: Rarity, Class, Archetype */}
                    <div className="flex items-center gap-2 text-xs">
                        {/* Rarity stars */}
                        <div className="flex items-center gap-0.5">
                            {Array.from({ length: rarityNum }).map((_, i) => (
                                // biome-ignore lint/suspicious/noArrayIndexKey: Static array of stars based on rarity count, order never changes
                                <span key={i} style={{ color: rarityColor }}>
                                    ★
                                </span>
                            ))}
                        </div>

                        <span className="text-muted-foreground/50">·</span>

                        {/* Class with icon */}
                        <div className="flex items-center gap-1">
                            <div className="flex h-4 w-4 items-center justify-center">
                                <Image alt={formatProfession(operator.profession)} className="opacity-60" height={14} src={`/api/cdn/upk/arts/ui/[uc]charcommon/icon_profession_${operator.profession.toLowerCase()}.png`} width={14} />
                            </div>
                            <span className="text-muted-foreground">{formatProfession(operator.profession)}</span>
                        </div>

                        <span className="text-muted-foreground/50">·</span>

                        {/* Archetype */}
                        <span className="truncate text-muted-foreground">{capitalize(formatSubProfession(operator.subProfessionId.toLowerCase()))}</span>
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
