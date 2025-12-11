"use client";

import { Maximize2, X } from "lucide-react";
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
            </div>
            <Separator />

            {isLoading ? (
                <div className="flex h-64 items-center justify-center">
                    <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
                </div>
            ) : skins.length > 0 ? (
                <AnimatedGroup className="space-y-6" preset="blur-slide">
                    {/* Main Viewer */}
                    <div className="grid gap-6 lg:grid-cols-[2fr,1fr]">
                        {/* Skin Image */}
                        <motion.div className="relative aspect-[3/4] overflow-hidden rounded-lg border border-border bg-card/50" layout>
                            {selectedSkinImage && <Image alt={selectedSkin?.displaySkin?.skinName ?? operator.name} className="object-contain" fill src={selectedSkinImage || "/placeholder.svg"} />}
                            <Button className="absolute top-2 right-2 bg-transparent" onClick={() => setIsFullscreen(true)} size="icon" variant="outline">
                                <Maximize2 className="h-4 w-4" />
                            </Button>
                        </motion.div>

                        {/* Skin Details */}
                        <div className="space-y-4">
                            <div className="rounded-lg border border-border bg-card/50 p-4">
                                <h3 className="font-semibold text-lg">{selectedSkin?.displaySkin?.skinName ?? "Default"}</h3>
                                {selectedSkin?.displaySkin?.content && <p className="mt-2 text-muted-foreground text-sm">{selectedSkin.displaySkin.content}</p>}
                                {selectedSkin?.displaySkin?.drawerList && selectedSkin.displaySkin.drawerList.length > 0 && (
                                    <div className="mt-3">
                                        <span className="text-muted-foreground text-xs">Artist</span>
                                        <p className="font-medium">{selectedSkin.displaySkin.drawerList.join(", ")}</p>
                                    </div>
                                )}
                                {selectedSkin?.displaySkin?.obtainApproach && (
                                    <div className="mt-3">
                                        <span className="text-muted-foreground text-xs">Obtain Method</span>
                                        <p className="font-medium">{selectedSkin.displaySkin.obtainApproach}</p>
                                    </div>
                                )}
                            </div>

                            {/* Chibi Placeholder */}
                            <div className="flex justify-center">
                                <ChibiViewerPlaceholder />
                            </div>
                        </div>
                    </div>

                    {/* Skin Selector */}
                    <div className="space-y-2">
                        <h4 className="font-semibold">Available Skins</h4>
                        <div className="flex flex-wrap gap-2">
                            {skins.map((skin) => (
                                <motion.button
                                    className={`relative h-16 w-16 overflow-hidden rounded-lg border-2 transition-colors ${selectedSkin?.skinId === skin.skinId ? "border-primary" : "border-border hover:border-primary/50"}`}
                                    key={skin.skinId}
                                    onClick={() => setSelectedSkin(skin)}
                                    whileHover={{ scale: 1.05 }}
                                    whileTap={{ scale: 0.95 }}
                                >
                                    <Image alt={skin.displaySkin?.skinName ?? "Skin"} className="object-cover" fill src={skin.images?.avatar ?? skin.images?.portrait ?? ""} />
                                </motion.button>
                            ))}
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
                        <Button className="absolute top-4 right-4 bg-transparent" onClick={() => setIsFullscreen(false)} size="icon" variant="outline">
                            <X className="h-4 w-4" />
                        </Button>
                    </div>
                </DialogContent>
            </Dialog>
        </div>
    );
}
