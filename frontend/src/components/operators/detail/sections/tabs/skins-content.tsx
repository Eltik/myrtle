"use client";

import { AnimatePresence, motion } from "framer-motion";
import { Calendar, Maximize2, Palette, User } from "lucide-react";
import Image from "next/image";
import { useEffect, useState } from "react";
import { Button } from "~/components/ui/shadcn/button";
import { Dialog, DialogContent, DialogTrigger } from "~/components/ui/shadcn/dialog";
import { ScrollArea, ScrollBar } from "~/components/ui/shadcn/scroll-area";
import { Skeleton } from "~/components/ui/shadcn/skeleton";
import { cn } from "~/lib/utils";
import type { Operator } from "~/types/api";
import type { Skin, SkinData } from "~/types/api/impl/skin";

interface SkinsContentProps {
    operator: Operator;
}

interface UISkin {
    id: string;
    name: string;
    image: string;
    thumbnail: string;
    displaySkin?: {
        skinName?: string;
        modelName?: string;
        drawerList?: string[];
        designerList?: string[];
        obtainApproach?: string;
    };
    isDefault: boolean;
}

export function SkinsContent({ operator }: SkinsContentProps) {
    const [skins, setSkins] = useState<UISkin[]>([]);
    const [selectedSkin, setSelectedSkin] = useState<string>("");
    const [isLoading, setIsLoading] = useState(true);
    const [imageLoading, setImageLoading] = useState(true);

    // Fetch skins
    useEffect(() => {
        const fetchSkins = async () => {
            setIsLoading(true);
            try {
                const res = await fetch("/api/static", {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ type: "skins", id: operator.id }),
                });
                const data = await res.json();

                if (data.skins) {
                    const formattedSkins = formatSkins(data.skins, operator);
                    setSkins(formattedSkins);
                    if (formattedSkins.length > 0) {
                        setSelectedSkin(formattedSkins[0]?.id ?? "");
                    }
                }
            } catch (error) {
                console.error("Failed to fetch skins:", error);
                // Create default skin entry - prefer full art (skin) over portrait
                const operatorId = operator.id ?? "";
                const skinPath = operator.skin ? `/api/cdn${operator.skin}` : null;
                const portraitPath = operator.portrait ? `/api/cdn${operator.portrait}` : null;
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

        if (operator.id) {
            fetchSkins();
        }
    }, [operator]); // Updated to use the entire operator object

    const selectedSkinData = skins.find((s) => s.id === selectedSkin);

    return (
        <div className="min-w-0 overflow-hidden p-4 md:p-6">
            {/* Header */}
            <div className="mb-6">
                <h2 className="font-semibold text-foreground text-xl">{operator.name} Skins</h2>
                <p className="text-muted-foreground text-sm">View available outfits and skins</p>
            </div>

            {isLoading ? (
                <div className="space-y-4">
                    <Skeleton className="aspect-3/4 max-h-[70vh] w-full max-w-md rounded-lg" />
                    <div className="flex gap-2">
                        {[1, 2, 3].map((i) => (
                            <Skeleton className="h-20 w-20 rounded-lg" key={i} />
                        ))}
                    </div>
                </div>
            ) : (
                <div className="grid min-w-0 gap-6 lg:grid-cols-[1fr,300px]">
                    {/* Main Image Viewer */}
                    <div className="relative">
                        <AnimatePresence mode="wait">
                            <motion.div animate={{ opacity: 1 }} className="relative aspect-3/4 max-h-[70vh] max-w-md overflow-hidden rounded-lg border border-border bg-card/30" exit={{ opacity: 0 }} initial={{ opacity: 0 }} key={selectedSkin} transition={{ duration: 0.2 }}>
                                {imageLoading && (
                                    <div className="absolute inset-0 flex items-center justify-center">
                                        <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary border-t-transparent" />
                                    </div>
                                )}
                                <Image alt={selectedSkinData?.name ?? "Skin"} className={cn("object-contain transition-opacity duration-300", imageLoading ? "opacity-0" : "opacity-100")} fill onLoad={() => setImageLoading(false)} priority src={selectedSkinData?.image ?? ""} />

                                {/* Fullscreen button */}
                                <Dialog>
                                    <DialogTrigger asChild>
                                        <Button className="absolute top-3 right-3" size="icon" variant="secondary">
                                            <Maximize2 className="h-4 w-4" />
                                        </Button>
                                    </DialogTrigger>
                                    <DialogContent className="max-h-[90vh] max-w-[90vw] p-0">
                                        <div className="relative h-[80vh] w-full">
                                            <Image alt={selectedSkinData?.name ?? "Skin"} className="object-contain" fill src={selectedSkinData?.image ?? ""} />
                                        </div>
                                    </DialogContent>
                                </Dialog>
                            </motion.div>
                        </AnimatePresence>
                    </div>

                    {/* Skin Details Panel */}
                    <div className="space-y-4">
                        <div className="rounded-lg border border-border bg-card/30 p-4">
                            <h3 className="mb-3 font-medium text-foreground">{selectedSkinData?.displaySkin?.skinName ?? selectedSkinData?.name ?? "Default"}</h3>

                            {selectedSkinData?.displaySkin ? (
                                <div className="space-y-3 text-sm">
                                    {selectedSkinData.displaySkin.drawerList && selectedSkinData.displaySkin.drawerList.length > 0 && (
                                        <div className="flex items-start gap-2">
                                            <Palette className="mt-0.5 h-4 w-4 text-muted-foreground" />
                                            <div>
                                                <div className="text-muted-foreground text-xs">Artist</div>
                                                <div className="text-foreground">{selectedSkinData.displaySkin.drawerList.join(", ")}</div>
                                            </div>
                                        </div>
                                    )}
                                    {selectedSkinData.displaySkin.obtainApproach && (
                                        <div className="flex items-start gap-2">
                                            <Calendar className="mt-0.5 h-4 w-4 text-muted-foreground" />
                                            <div>
                                                <div className="text-muted-foreground text-xs">Obtain</div>
                                                <div className="text-foreground">{selectedSkinData.displaySkin.obtainApproach}</div>
                                            </div>
                                        </div>
                                    )}
                                </div>
                            ) : (
                                <p>N/A</p>
                            )}
                        </div>

                        {/* Chibi Viewer Placeholder */}
                        <div className="rounded-lg border border-border border-dashed bg-secondary/20 p-6 text-center">
                            <User className="mx-auto mb-2 h-8 w-8 text-muted-foreground" />
                            <p className="text-muted-foreground text-sm">Chibi Viewer Coming Soon</p>
                        </div>
                    </div>
                </div>
            )}

            {/* Skin Selector */}
            {!isLoading && skins.length > 0 && (
                <div className="mt-6">
                    <h3 className="mb-3 font-medium text-foreground text-sm">Available Skins</h3>
                    <ScrollArea className="w-full">
                        <div className="flex gap-3 pb-2">
                            {skins.map((skin) => (
                                <button
                                    className={cn("relative h-24 w-24 shrink-0 overflow-hidden rounded-lg border-2 transition-all hover:scale-105", selectedSkin === skin.id ? "border-primary shadow-lg" : "border-border/50 hover:border-primary/50")}
                                    key={skin.id}
                                    onClick={() => {
                                        setImageLoading(true);
                                        setSelectedSkin(skin.id);
                                    }}
                                    type="button"
                                >
                                    <Image alt={skin.name} className="object-cover" fill src={skin.thumbnail || "/placeholder.svg"} />
                                    <div className="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/90 via-black/60 to-transparent px-2 pt-4 pb-1.5">
                                        <span className="line-clamp-1 font-medium text-white text-xs drop-shadow-sm">{skin.name}</span>
                                    </div>
                                </button>
                            ))}
                        </div>
                        <ScrollBar orientation="horizontal" />
                    </ScrollArea>
                </div>
            )}
        </div>
    );
}

function formatSkins(skinData: SkinData | Skin[], operator: Operator): UISkin[] {
    const skins: UISkin[] = [];
    const operatorId = operator.id ?? "";

    // Prefer full character art (skin) over portrait (headshot)
    // skin: /upk/chararts/{id}/{id}_{1|2}.png (large full art)
    // portrait: /upk/arts/charportraits/{pack}/{id}_{1|2}.png (small headshot)
    const skinPath = operator.skin ? `/api/cdn${operator.skin}` : null;
    const portraitPath = operator.portrait ? `/api/cdn${operator.portrait}` : null;

    // Use full character art if available, otherwise fall back to portrait
    const basePath = skinPath ?? portraitPath;
    const e0Path = basePath?.replace(/_2\.png$/, "_1.png") ?? `/api/cdn/upk/chararts/${operatorId}/${operatorId}_1.png`;
    const e2Path = basePath?.replace(/_1\.png$/, "_2.png") ?? `/api/cdn/upk/chararts/${operatorId}/${operatorId}_2.png`;

    // Add default skin (E0/E1 art)
    skins.push({
        id: `${operatorId}_default`,
        name: "Default",
        image: e0Path,
        thumbnail: e0Path,
        isDefault: true,
    });

    // Add E2 art if available (phases > 2 means E2 exists)
    if (operator.phases.length > 2) {
        skins.push({
            id: `${operatorId}_e2`,
            name: "Elite 2",
            image: e2Path,
            thumbnail: e2Path,
            isDefault: false,
        });
    }

    // Process additional skins from API
    if (Array.isArray(skinData)) {
        skinData.forEach((skin) => {
            // Use skinId (standard Skin type) - EnrichedSkin also has skinId via extension
            const skinIdentifier = skin.skinId;
            // Skip default skins (those with #1 or #2 suffixes which are E0/E1 and E2 arts)
            if (skinIdentifier && !skinIdentifier.endsWith("#1") && !skinIdentifier.endsWith("#2")) {
                // Format the skin path for CDN
                // skinId format: "char_002_amiya@epoque#4" -> file: "char_002_amiya_epoque#4.png"
                // Replace @ with _ and encode # as %23 (# is a URL fragment identifier and won't be sent to server)
                const formattedSkinId = skinIdentifier.replace(/@/g, "_").replace(/#/g, "%23");

                skins.push({
                    id: skinIdentifier,
                    name: skin.displaySkin?.skinName ?? "Outfit",
                    image: `/api/cdn/upk/skinpack/${operatorId}/${formattedSkinId}.png`,
                    thumbnail: `/api/cdn/upk/skinpack/${operatorId}/${formattedSkinId}.png`,
                    displaySkin: skin.displaySkin
                        ? {
                              skinName: skin.displaySkin.skinName ?? undefined,
                              modelName: skin.displaySkin.modelName,
                              drawerList: skin.displaySkin.drawerList,
                              designerList: skin.displaySkin.designerList ?? undefined,
                              obtainApproach: skin.displaySkin.obtainApproach ?? undefined,
                          }
                        : undefined,
                    isDefault: false,
                });
            }
        });
    }

    return skins;
}
