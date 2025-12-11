"use client";

import { useEffect, useState } from "react";
import { motion } from "motion/react";
import Image from "next/image";
import { Maximize2, X } from "lucide-react";
import type { Operator, EnrichedSkin } from "~/types/api";
import { Separator } from "~/components/ui/shadcn/separator";
import { AnimatedGroup } from "~/components/ui/motion-primitives/animated-group";
import { Button } from "~/components/ui/shadcn/button";
import { Dialog, DialogContent, DialogTitle } from "~/components/ui/shadcn/dialog";
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
                <h2 className="text-2xl font-bold md:text-3xl">{operator.name} Skins</h2>
            </div>
            <Separator />

            {isLoading ? (
                <div className="flex h-64 items-center justify-center">
                    <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
                </div>
            ) : skins.length > 0 ? (
                <AnimatedGroup preset="blur-slide" className="space-y-6">
                    {/* Main Viewer */}
                    <div className="grid gap-6 lg:grid-cols-[2fr,1fr]">
                        {/* Skin Image */}
                        <motion.div layout className="relative aspect-[3/4] overflow-hidden rounded-lg border border-border bg-card/50">
                            {selectedSkinImage && <Image src={selectedSkinImage || "/placeholder.svg"} alt={selectedSkin?.displaySkin?.skinName ?? operator.name} fill className="object-contain" />}
                            <Button variant="outline" size="icon" className="absolute right-2 top-2 bg-transparent" onClick={() => setIsFullscreen(true)}>
                                <Maximize2 className="h-4 w-4" />
                            </Button>
                        </motion.div>

                        {/* Skin Details */}
                        <div className="space-y-4">
                            <div className="rounded-lg border border-border bg-card/50 p-4">
                                <h3 className="text-lg font-semibold">{selectedSkin?.displaySkin?.skinName ?? "Default"}</h3>
                                {selectedSkin?.displaySkin?.content && <p className="mt-2 text-sm text-muted-foreground">{selectedSkin.displaySkin.content}</p>}
                                {selectedSkin?.displaySkin?.drawerList && selectedSkin.displaySkin.drawerList.length > 0 && (
                                    <div className="mt-3">
                                        <span className="text-xs text-muted-foreground">Artist</span>
                                        <p className="font-medium">{selectedSkin.displaySkin.drawerList.join(", ")}</p>
                                    </div>
                                )}
                                {selectedSkin?.displaySkin?.obtainApproach && (
                                    <div className="mt-3">
                                        <span className="text-xs text-muted-foreground">Obtain Method</span>
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
                                    key={skin.skinId}
                                    whileHover={{ scale: 1.05 }}
                                    whileTap={{ scale: 0.95 }}
                                    onClick={() => setSelectedSkin(skin)}
                                    className={`relative h-16 w-16 overflow-hidden rounded-lg border-2 transition-colors ${selectedSkin?.skinId === skin.skinId ? "border-primary" : "border-border hover:border-primary/50"}`}
                                >
                                    <Image src={skin.images?.avatar ?? skin.images?.portrait ?? ""} alt={skin.displaySkin?.skinName ?? "Skin"} fill className="object-cover" />
                                </motion.button>
                            ))}
                        </div>
                    </div>
                </AnimatedGroup>
            ) : (
                <p className="text-muted-foreground">No skins available for this operator.</p>
            )}

            {/* Fullscreen Dialog */}
            <Dialog open={isFullscreen} onOpenChange={setIsFullscreen}>
                <DialogContent className="max-w-[95vw] max-h-[95vh] p-0 bg-background/95 backdrop-blur-md">
                    <DialogTitle className="sr-only">{selectedSkin?.displaySkin?.skinName ?? operator.name} Skin Preview</DialogTitle>
                    <div className="relative h-[90vh] w-full">
                        {selectedSkinImage && <Image src={selectedSkinImage || "/placeholder.svg"} alt={selectedSkin?.displaySkin?.skinName ?? operator.name} fill className="object-contain" />}
                        <Button variant="outline" size="icon" className="absolute right-4 top-4 bg-transparent" onClick={() => setIsFullscreen(false)}>
                            <X className="h-4 w-4" />
                        </Button>
                    </div>
                </DialogContent>
            </Dialog>
        </div>
    );
}
