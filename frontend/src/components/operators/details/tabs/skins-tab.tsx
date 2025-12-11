"use client";

import { Check, Maximize2, X } from "lucide-react";
import { motion } from "motion/react";
import Image from "next/image";
import { useEffect, useState } from "react";
import { AnimatedGroup } from "~/components/ui/motion-primitives/animated-group";
import { Button } from "~/components/ui/shadcn/button";
import { Dialog, DialogContent, DialogTitle } from "~/components/ui/shadcn/dialog";
import { Separator } from "~/components/ui/shadcn/separator";
import type { EnrichedSkin, Operator } from "~/types/api";
import { ChibiViewerPlaceholder } from "../chibi-viewer-placeholder";

interface SkinsTabProps {
    operator: Operator;
}

export function SkinsTab({ operator }: SkinsTabProps) {
    const [skins, setSkins] = useState<EnrichedSkin[]>([]);
    const [selectedSkin, setSelectedSkin] = useState<EnrichedSkin | null>(null);
    const [isFullscreen, setIsFullscreen] = useState(false);
    const [isLoading, setIsLoading] = useState(true);

    // Fetch skins
    useEffect(() => {
        const fetchSkins = async () => {
            try {
                const response = await fetch("/api/static", {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ type: "skins", id: operator.id }),
                });
                const data = (await response.json()) as { skins: EnrichedSkin[] };
                const fetchedSkins = data.skins ?? [];
                setSkins(fetchedSkins);
                if (fetchedSkins.length > 0) {
                    setSelectedSkin(fetchedSkins[0]!);
                }
            } catch (error) {
                console.error("Failed to fetch skins:", error);
            } finally {
                setIsLoading(false);
            }
        };
        void fetchSkins();
    }, [operator.id]);

    const selectedSkinImage = selectedSkin?.images?.skin ?? selectedSkin?.images?.portrait ?? "";

    return (
        <div className="w-full space-y-6">
            <div className="flex items-center justify-between">
                <h2 className="font-bold text-2xl md:text-3xl">{operator.name} Skins</h2>
                <span className="text-muted-foreground text-sm">{skins.length} available</span>
            </div>
            <Separator />

            {isLoading ? (
                <div className="flex h-64 items-center justify-center">
                    <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
                </div>
            ) : skins.length > 0 ? (
                <AnimatedGroup className="space-y-6" preset="blur-slide">
                    {/* Skin Selector - Horizontal Scroll */}
                    <div className="space-y-2">
                        <h4 className="font-semibold text-sm">Select Skin</h4>
                        <div className="flex gap-3 overflow-x-auto pb-2">
                            {skins.map((skin) => {
                                const isSelected = selectedSkin?.skinId === skin.skinId;
                                return (
                                    <motion.button
                                        className={`relative flex-shrink-0 overflow-hidden rounded-lg border-2 transition-all ${isSelected ? "border-primary shadow-lg shadow-primary/20" : "border-border hover:border-primary/50"}`}
                                        key={skin.skinId}
                                        onClick={() => setSelectedSkin(skin)}
                                        whileHover={{ scale: 1.02 }}
                                        whileTap={{ scale: 0.98 }}
                                    >
                                        <div className="relative h-24 w-24 overflow-hidden">
                                            <Image alt={skin.displaySkin?.skinName ?? "Skin"} className="object-cover" fill src={skin.images?.avatar ?? skin.images?.portrait ?? ""} />
                                        </div>
                                        {isSelected && (
                                            <div className="absolute top-1 right-1 rounded-full bg-primary p-1">
                                                <Check className="h-3 w-3 text-primary-foreground" />
                                            </div>
                                        )}
                                        <div className="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/80 to-transparent px-2 py-1">
                                            <span className="line-clamp-1 text-white text-xs">{skin.displaySkin?.skinName ?? "Default"}</span>
                                        </div>
                                    </motion.button>
                                );
                            })}
                        </div>
                    </div>

                    {/* Main Viewer */}
                    <div className="grid gap-6 lg:grid-cols-[2fr,1fr]">
                        {/* Skin Image */}
                        <motion.div className="relative aspect-[3/4] overflow-hidden rounded-lg border border-border bg-gradient-to-b from-muted/30 to-muted/50" layout>
                            {selectedSkinImage && <Image alt={selectedSkin?.displaySkin?.skinName ?? operator.name} className="object-contain" fill src={selectedSkinImage || "/placeholder.svg"} />}
                            <Button className="absolute top-3 right-3 bg-background/80 backdrop-blur-sm" onClick={() => setIsFullscreen(true)} size="icon" variant="outline">
                                <Maximize2 className="h-4 w-4" />
                            </Button>
                        </motion.div>

                        {/* Skin Details */}
                        <div className="space-y-4">
                            <div className="rounded-lg border border-border bg-card/50 p-4">
                                <h3 className="font-semibold text-xl">{selectedSkin?.displaySkin?.skinName ?? "Default"}</h3>

                                {selectedSkin?.displaySkin?.content && <p className="mt-2 text-muted-foreground text-sm leading-relaxed">{selectedSkin.displaySkin.content}</p>}

                                <div className="mt-4 space-y-3">
                                    {selectedSkin?.displaySkin?.drawerList && selectedSkin.displaySkin.drawerList.length > 0 && (
                                        <div>
                                            <span className="text-muted-foreground text-xs uppercase tracking-wider">Artist</span>
                                            <p className="font-medium">{selectedSkin.displaySkin.drawerList.join(", ")}</p>
                                        </div>
                                    )}

                                    {selectedSkin?.displaySkin?.skinGroupName && (
                                        <div>
                                            <span className="text-muted-foreground text-xs uppercase tracking-wider">Collection</span>
                                            <p className="font-medium">{selectedSkin.displaySkin.skinGroupName}</p>
                                        </div>
                                    )}

                                    {selectedSkin?.displaySkin?.obtainApproach && (
                                        <div>
                                            <span className="text-muted-foreground text-xs uppercase tracking-wider">Obtain Method</span>
                                            <p className="font-medium">{selectedSkin.displaySkin.obtainApproach}</p>
                                        </div>
                                    )}

                                    {selectedSkin?.displaySkin?.usage && (
                                        <div>
                                            <span className="text-muted-foreground text-xs uppercase tracking-wider">Usage</span>
                                            <p className="text-muted-foreground text-sm">{selectedSkin.displaySkin.usage}</p>
                                        </div>
                                    )}

                                    {selectedSkin?.displaySkin?.description && (
                                        <div>
                                            <span className="text-muted-foreground text-xs uppercase tracking-wider">Description</span>
                                            <p className="text-muted-foreground text-sm leading-relaxed">{selectedSkin.displaySkin.description}</p>
                                        </div>
                                    )}
                                </div>
                            </div>

                            {/* Chibi Placeholder */}
                            <div className="flex justify-center">
                                <ChibiViewerPlaceholder />
                            </div>
                        </div>
                    </div>
                </AnimatedGroup>
            ) : (
                <p className="text-muted-foreground">No skins available for this operator.</p>
            )}

            {/* Fullscreen Dialog */}
            <Dialog onOpenChange={setIsFullscreen} open={isFullscreen}>
                <DialogContent className="max-h-[95vh] max-w-[95vw] bg-background/95 p-0 backdrop-blur-md">
                    <DialogTitle className="sr-only">{selectedSkin?.displaySkin?.skinName ?? operator.name} Skin Preview</DialogTitle>
                    <div className="relative h-[90vh] w-full">
                        {selectedSkinImage && <Image alt={selectedSkin?.displaySkin?.skinName ?? operator.name} className="object-contain" fill src={selectedSkinImage || "/placeholder.svg"} />}
                        <Button className="absolute top-4 right-4 bg-background/80 backdrop-blur-sm" onClick={() => setIsFullscreen(false)} size="icon" variant="outline">
                            <X className="h-4 w-4" />
                        </Button>
                    </div>
                </DialogContent>
            </Dialog>
        </div>
    );
}
