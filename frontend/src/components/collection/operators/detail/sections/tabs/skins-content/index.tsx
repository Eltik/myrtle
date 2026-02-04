"use client";

import { Brush, Calendar, ChevronRight, FileText, Maximize2, MessageCircle, Palette, Sparkles, Tag } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import Image from "next/image";
import { memo, useCallback, useEffect, useMemo, useState } from "react";
import { Disclosure, DisclosureContent, DisclosureTrigger } from "~/components/ui/motion-primitives/disclosure";
import { Button } from "~/components/ui/shadcn/button";
import { ScrollArea } from "~/components/ui/shadcn/scroll-area";
import { Skeleton } from "~/components/ui/shadcn/skeleton";
import { Tooltip, TooltipContent, TooltipTrigger } from "~/components/ui/shadcn/tooltip";
import { cn } from "~/lib/utils";
import type { Operator } from "~/types/api";
import type { ChibiCharacter } from "~/types/api/impl/chibi";
import { DynamicChibiViewer } from "../chibi-viewer/dynamic";
import { ColorTagText } from "./impl/color-tag-parser";
import { formatSkinsForOperator } from "./impl/helpers";
import { SkinViewerDialog } from "./impl/skin-viewer-dialog";
import type { UISkin } from "./impl/types";

interface SkinsContentProps {
    operator: Operator;
}

// Helper component for displaying color swatches
interface ColorSwatchesProps {
    colors: string[];
}

const ColorSwatches = memo(function ColorSwatches({ colors }: ColorSwatchesProps) {
    const validColors = colors.filter((c) => c?.startsWith("#"));

    if (validColors.length === 0) return null;

    return (
        <div className="flex flex-wrap items-center gap-1.5">
            {validColors.map((color) => (
                <Tooltip key={color}>
                    <TooltipTrigger asChild>
                        <div className="h-6 w-6 cursor-pointer rounded-md border border-border/50 shadow-sm transition-transform hover:scale-110" style={{ backgroundColor: color }} />
                    </TooltipTrigger>
                    <TooltipContent>
                        <span className="font-mono text-xs">{color.toUpperCase()}</span>
                    </TooltipContent>
                </Tooltip>
            ))}
        </div>
    );
});

// Helper component for consistent detail row styling
interface DetailRowProps {
    icon: React.ElementType;
    label: string;
    children: React.ReactNode;
}

const DetailRow = memo(function DetailRow({ icon: Icon, label, children }: DetailRowProps) {
    return (
        <div className="flex items-start gap-2">
            <Icon className="mt-0.5 h-4 w-4 shrink-0 text-muted-foreground" />
            <div className="min-w-0 flex-1">
                <div className="text-muted-foreground text-xs">{label}</div>
                <div className="text-foreground text-sm">{children}</div>
            </div>
        </div>
    );
});

// Helper component for collapsible text sections
interface CollapsibleSectionProps {
    icon: React.ElementType;
    label: string;
    children: React.ReactNode;
}

const CollapsibleSection = memo(function CollapsibleSection({ icon: Icon, label, children }: CollapsibleSectionProps) {
    const [isOpen, setIsOpen] = useState(false);

    return (
        <Disclosure className="rounded-md border border-border/50 bg-secondary/20" onOpenChange={setIsOpen} open={isOpen} transition={{ duration: 0.2, ease: "easeInOut" }}>
            <DisclosureTrigger>
                <div className="flex w-full cursor-pointer items-center justify-between px-3 py-2 hover:bg-secondary/30">
                    <div className="flex items-center gap-2">
                        <Icon className="h-4 w-4 text-muted-foreground" />
                        <span className="text-foreground text-sm">{label}</span>
                    </div>
                    <ChevronRight className={cn("h-4 w-4 text-muted-foreground transition-transform duration-200", isOpen && "rotate-90")} />
                </div>
            </DisclosureTrigger>
            <DisclosureContent>
                <div className="border-border/50 border-t px-3 py-2">{children}</div>
            </DisclosureContent>
        </Disclosure>
    );
});

export const SkinsContent = memo(function SkinsContent({ operator }: SkinsContentProps) {
    const [skins, setSkins] = useState<UISkin[]>([]);
    const [selectedSkin, setSelectedSkin] = useState<string>("");
    const [isLoading, setIsLoading] = useState(true);
    const [imageLoading, setImageLoading] = useState(true);
    const [chibiData, setChibiData] = useState<ChibiCharacter | null>(null);

    const operatorId = operator.id ?? "";
    const operatorSkin = operator.skin;
    const operatorPortrait = operator.portrait;
    const phasesLength = operator.phases.length;

    useEffect(() => {
        const fetchData = async () => {
            setIsLoading(true);

            // Fetch skins and chibi data in parallel
            const skinsPromise = fetch("/api/static", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ type: "skins", id: operatorId }),
            }).then((res) => res.json());

            const chibiPromise = fetch("/api/static", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ type: "chibis", id: operatorId }),
            })
                .then((res) => res.json())
                .catch(() => null);

            try {
                const [skinsData, chibiResponse] = await Promise.all([skinsPromise, chibiPromise]);

                if (skinsData.skins) {
                    const formattedSkins = formatSkinsForOperator(skinsData.skins, operatorId, operatorSkin ?? undefined, operatorPortrait, phasesLength);
                    setSkins(formattedSkins);
                    if (formattedSkins.length > 0) {
                        setSelectedSkin(formattedSkins[0]?.id ?? "");
                    }
                }

                if (chibiResponse?.chibi) {
                    setChibiData(chibiResponse.chibi);
                }
            } catch (error) {
                console.error("Failed to fetch skins:", error);
                // Create default skin entry - prefer full art (skin) over portrait
                const skinPath = operatorSkin ? `/api/cdn${operatorSkin}` : null;
                const portraitPath = operatorPortrait ? `/api/cdn${operatorPortrait}` : null;
                const basePath = skinPath ?? portraitPath;
                const e0Path = basePath?.replace(/_2\.png$/, "_1.png") ?? `/api/cdn/upk/chararts/${operatorId}/${operatorId}_1.png`;
                const defaultSkin: UISkin = {
                    id: operatorId,
                    name: "Default",
                    image: e0Path,
                    thumbnail: e0Path,
                    isDefault: true,
                };
                setSkins([defaultSkin]);
                setSelectedSkin(defaultSkin.id);
            }
            setIsLoading(false);
        };

        if (operatorId) {
            fetchData();
        }
    }, [operatorId, operatorSkin, operatorPortrait, phasesLength]);

    const selectedSkinData = useMemo(() => skins.find((s) => s.id === selectedSkin), [skins, selectedSkin]);

    // Derive the chibi skin name from the selected skin
    const chibiSkinName = useMemo(() => {
        if (!selectedSkin) return "default";

        // For default/E2 skins, use "default"
        if (selectedSkin.endsWith("_default") || selectedSkin.endsWith("_e2")) {
            return "default";
        }

        // For custom skins, extract the skin name from the skin ID
        // Format: char_xxx_name@skingroup#version -> skingroup#version
        const atIndex = selectedSkin.indexOf("@");
        if (atIndex !== -1) {
            return selectedSkin.slice(atIndex + 1);
        }

        return "default";
    }, [selectedSkin]);

    const handleSkinSelect = useCallback((skinId: string) => {
        setImageLoading(true);
        setSelectedSkin(skinId);
    }, []);

    return (
        <div className="min-w-0 overflow-hidden p-4 md:p-6">
            {/* Header */}
            <div className="mb-6">
                <h2 className="font-semibold text-foreground text-xl">{operator.name} Skins</h2>
                <p className="text-muted-foreground text-sm">View available outfits and skins</p>
            </div>

            {isLoading ? (
                <div className="grid min-w-0 gap-6 lg:grid-cols-[1fr,300px]">
                    <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:gap-4">
                        <Skeleton className="aspect-3/4 max-h-[70vh] w-full flex-1 rounded-lg" />
                        <div className="hidden shrink-0 flex-col gap-3 lg:flex">
                            {[1, 2, 3].map((i) => (
                                <Skeleton className="h-24 w-24 rounded-lg" key={i} />
                            ))}
                        </div>
                        <div className="flex gap-2 lg:hidden">
                            {[1, 2, 3].map((i) => (
                                <Skeleton className="h-20 w-20 rounded-lg" key={i} />
                            ))}
                        </div>
                    </div>
                    <div className="space-y-4">
                        <Skeleton className="h-40 w-full rounded-lg" />
                        <Skeleton className="h-32 w-full rounded-lg" />
                    </div>
                </div>
            ) : (
                <div className="grid min-w-0 gap-6 lg:grid-cols-[1fr,300px]">
                    {/* Left column: Image Viewer + Skin Selector */}
                    <div className="flex min-w-0 flex-col gap-4 overflow-hidden">
                        <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:gap-4">
                            {/* Main Image Viewer */}
                            <div className="relative min-w-0 flex-1">
                                <AnimatePresence mode="wait">
                                    <motion.div animate={{ opacity: 1 }} className="relative aspect-3/4 max-h-[70vh] w-full overflow-hidden rounded-lg border border-border bg-card/30 p-3" exit={{ opacity: 0 }} initial={{ opacity: 0 }} key={selectedSkin} transition={{ duration: 0.2 }}>
                                        {imageLoading && (
                                            <div className="absolute inset-0 flex items-center justify-center">
                                                <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary border-t-transparent" />
                                            </div>
                                        )}
                                        <Image alt={selectedSkinData?.name ?? "Skin"} className={cn("object-contain transition-opacity duration-300", imageLoading ? "opacity-0" : "opacity-100")} fill onLoad={() => setImageLoading(false)} priority src={selectedSkinData?.image ?? ""} />

                                        {/* Fullscreen button */}
                                        <SkinViewerDialog imageSrc={selectedSkinData?.image ?? ""} skinName={selectedSkinData?.name ?? "Skin"}>
                                            <Button className="absolute top-3 right-3" size="icon" variant="secondary">
                                                <Maximize2 className="h-4 w-4" />
                                            </Button>
                                        </SkinViewerDialog>
                                    </motion.div>
                                </AnimatePresence>
                            </div>

                            {/* Skin Selector - Vertical on desktop */}
                            {skins.length > 0 && (
                                <div className="hidden shrink-0 flex-col lg:flex">
                                    <h3 className="mb-3 shrink-0 font-medium text-foreground text-sm">Available Skins</h3>
                                    <ScrollArea className="h-[calc(70vh-3rem)]">
                                        <div className="flex flex-col gap-3 p-1 pr-3">
                                            {skins.map((skin) => (
                                                <button
                                                    className={cn("relative h-24 w-24 shrink-0 overflow-hidden rounded-lg border-2 transition-all hover:scale-105", selectedSkin === skin.id ? "border-primary shadow-lg" : "border-border/50 hover:border-primary/50")}
                                                    key={skin.id}
                                                    onClick={() => handleSkinSelect(skin.id)}
                                                    type="button"
                                                >
                                                    <Image alt={skin.name} className="object-cover" fill src={skin.thumbnail || "/placeholder.svg"} />
                                                    <div className="absolute inset-x-0 bottom-0 bg-linear-to-t from-black/90 via-black/60 to-transparent px-2 pt-4 pb-1.5">
                                                        <span className="line-clamp-1 font-medium text-white text-xs drop-shadow-sm">{skin.name}</span>
                                                    </div>
                                                </button>
                                            ))}
                                        </div>
                                    </ScrollArea>
                                </div>
                            )}
                        </div>

                        {/* Skin Selector - Horizontal on mobile */}
                        {skins.length > 0 && (
                            <div className="lg:hidden">
                                <h3 className="mb-3 font-medium text-foreground text-sm">Available Skins</h3>
                                <div
                                    className="overflow-x-auto overflow-y-hidden whitespace-nowrap pb-2.5"
                                    style={{
                                        WebkitOverflowScrolling: "touch",
                                    }}
                                >
                                    <div className="inline-flex gap-3 p-1">
                                        {skins.map((skin) => (
                                            <button
                                                className={cn("relative inline-block h-24 w-24 overflow-hidden rounded-lg border-2 transition-all hover:scale-105", selectedSkin === skin.id ? "border-primary shadow-lg" : "border-border/50 hover:border-primary/50")}
                                                key={skin.id}
                                                onClick={() => handleSkinSelect(skin.id)}
                                                type="button"
                                            >
                                                <Image alt={skin.name} className="object-cover" fill src={skin.thumbnail || "/placeholder.svg"} />
                                                <div className="absolute inset-x-0 bottom-0 bg-linear-to-t from-black/90 via-black/60 to-transparent px-2 pt-4 pb-1.5">
                                                    <span className="line-clamp-1 font-medium text-white text-xs drop-shadow-sm">{skin.name}</span>
                                                </div>
                                            </button>
                                        ))}
                                    </div>
                                </div>
                            </div>
                        )}
                    </div>

                    {/* Skin Details Panel */}
                    <div className="space-y-4">
                        <div className="rounded-lg border border-border bg-card/30 p-4">
                            <h3 className="mb-3 font-medium text-foreground">{selectedSkinData?.displaySkin?.skinName ?? selectedSkinData?.name ?? "Default"}</h3>

                            {selectedSkinData?.displaySkin ? (
                                <div className="space-y-3">
                                    {/* Color Palette */}
                                    {selectedSkinData.displaySkin.colorList && selectedSkinData.displaySkin.colorList.length > 0 && (
                                        <DetailRow icon={Palette} label="Color Palette">
                                            <ColorSwatches colors={selectedSkinData.displaySkin.colorList} />
                                        </DetailRow>
                                    )}

                                    {/* Artist */}
                                    {selectedSkinData.displaySkin.drawerList && selectedSkinData.displaySkin.drawerList.length > 0 && (
                                        <DetailRow icon={Brush} label="Artist">
                                            {selectedSkinData.displaySkin.drawerList.join(", ")}
                                        </DetailRow>
                                    )}

                                    {/* Designer */}
                                    {selectedSkinData.displaySkin.designerList && selectedSkinData.displaySkin.designerList.length > 0 && (
                                        <DetailRow icon={Sparkles} label="Designer">
                                            {selectedSkinData.displaySkin.designerList.join(", ")}
                                        </DetailRow>
                                    )}

                                    {/* Collection/Skin Group */}
                                    {selectedSkinData.displaySkin.skinGroupName && (
                                        <DetailRow icon={Tag} label="Collection">
                                            {selectedSkinData.displaySkin.skinGroupName}
                                        </DetailRow>
                                    )}

                                    {/* Obtain Approach */}
                                    {selectedSkinData.displaySkin.obtainApproach && (
                                        <DetailRow icon={Calendar} label="Obtain">
                                            {selectedSkinData.displaySkin.obtainApproach}
                                        </DetailRow>
                                    )}
                                </div>
                            ) : (
                                <p className="text-muted-foreground text-sm">Default operator appearance</p>
                            )}
                        </div>

                        {/* Collapsible Text Sections */}
                        {selectedSkinData?.displaySkin && (
                            <div className="space-y-2">
                                {/* Description */}
                                {selectedSkinData.displaySkin.description && (
                                    <CollapsibleSection icon={FileText} label="Description">
                                        <p className="text-muted-foreground text-sm leading-relaxed">{selectedSkinData.displaySkin.description}</p>
                                    </CollapsibleSection>
                                )}

                                {/* Dialog */}
                                {selectedSkinData.displaySkin.dialog && (
                                    <CollapsibleSection icon={MessageCircle} label="Dialog">
                                        <p className="text-muted-foreground text-sm italic leading-relaxed">"{selectedSkinData.displaySkin.dialog}"</p>
                                    </CollapsibleSection>
                                )}

                                {/* Content (with color tag parsing) */}
                                {selectedSkinData.displaySkin.content && (
                                    <CollapsibleSection icon={FileText} label="Content">
                                        <ColorTagText className="text-sm leading-relaxed" text={selectedSkinData.displaySkin.content} />
                                    </CollapsibleSection>
                                )}

                                {/* Usage */}
                                {selectedSkinData.displaySkin.usage && (
                                    <CollapsibleSection icon={Sparkles} label="Usage">
                                        <p className="text-muted-foreground text-sm leading-relaxed">{selectedSkinData.displaySkin.usage}</p>
                                    </CollapsibleSection>
                                )}
                            </div>
                        )}

                        {/* Chibi Viewer - Dynamically loaded to reduce bundle size */}
                        {chibiData && <DynamicChibiViewer chibi={chibiData} skinName={chibiSkinName} />}
                    </div>
                </div>
            )}
        </div>
    );
});
