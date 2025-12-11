"use client";

import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "motion/react";
import Image from "next/image";
import { Maximize2, X } from "lucide-react";
import type { Operator, EnrichedSkin } from "~/types/api";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Button } from "~/components/ui/shadcn/button";
import { Dialog, DialogContent, DialogClose } from "~/components/ui/shadcn/dialog";
import { ChibiViewer } from "../components/chibi-viewer";
import { cn } from "~/lib/utils";

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
                if (response.ok) {
                    const data = (await response.json()) as { skins: EnrichedSkin[] };
                    setSkins(data.skins ?? []);
                    if (data.skins && data.skins.length > 0) {
                        setSelectedSkin(data.skins[0] ?? null);
                    }
                }
            } catch (error) {
                console.error("Failed to fetch skins:", error);
            } finally {
                setIsLoading(false);
            }
        };
        void fetchSkins();
    }, [operator.id]);

    if (isLoading) {
        return (
            <div className="flex h-64 items-center justify-center">
                <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary border-t-transparent" />
            </div>
        );
    }

    return (
        <div className="space-y-6">
            <InView
                variants={{
                    hidden: { opacity: 0, y: 20 },
                    visible: { opacity: 1, y: 0 },
                }}
                transition={{ duration: 0.4 }}
            >
                <div className="rounded-xl border bg-card/50 p-6 backdrop-blur-sm">
                    <h2 className="mb-6 text-xl font-semibold">{operator.name} Skins</h2>

                    <div className="grid gap-6 lg:grid-cols-[2fr,1fr]">
                        {/* Main skin viewer */}
                        <div className="relative aspect-[3/4] overflow-hidden rounded-lg bg-muted">
                            {selectedSkin?.images?.skin && (
                                <>
                                    <Image src={selectedSkin.images.skin || "/placeholder.svg"} alt={selectedSkin.displaySkin.skinName ?? "Skin"} fill className="object-contain" unoptimized />
                                    <Button variant="secondary" size="icon" className="absolute right-2 top-2" onClick={() => setIsFullscreen(true)}>
                                        <Maximize2 className="h-4 w-4" />
                                    </Button>
                                </>
                            )}
                        </div>

                        {/* Skin details */}
                        <div className="space-y-4">
                            <AnimatePresence mode="wait">
                                {selectedSkin && (
                                    <motion.div key={selectedSkin.skinId} initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="space-y-4">
                                        <div>
                                            <h3 className="font-semibold">{selectedSkin.displaySkin.skinName ?? "Default"}</h3>
                                            <p className="text-sm text-muted-foreground">{selectedSkin.displaySkin.skinGroupName}</p>
                                        </div>

                                        {selectedSkin.displaySkin.content && (
                                            <div className="rounded-lg bg-muted/50 p-3">
                                                <p className="text-sm text-muted-foreground">{selectedSkin.displaySkin.content}</p>
                                            </div>
                                        )}

                                        {selectedSkin.displaySkin.drawerList && selectedSkin.displaySkin.drawerList.length > 0 && (
                                            <div>
                                                <span className="text-sm text-muted-foreground">Artist: </span>
                                                <span className="text-sm">{selectedSkin.displaySkin.drawerList.join(", ")}</span>
                                            </div>
                                        )}

                                        {selectedSkin.displaySkin.obtainApproach && (
                                            <div>
                                                <span className="text-sm text-muted-foreground">Obtain: </span>
                                                <span className="text-sm">{selectedSkin.displaySkin.obtainApproach}</span>
                                            </div>
                                        )}
                                    </motion.div>
                                )}
                            </AnimatePresence>
                        </div>
                    </div>

                    {/* Skin selector */}
                    <div className="mt-6">
                        <h4 className="mb-3 text-sm font-medium text-muted-foreground">Available Skins</h4>
                        <div className="flex flex-wrap gap-2">
                            {skins.map((skin) => (
                                <button key={skin.skinId} onClick={() => setSelectedSkin(skin)} className={cn("relative h-16 w-16 overflow-hidden rounded-lg border-2 transition-all", selectedSkin?.skinId === skin.skinId ? "border-primary" : "border-transparent hover:border-muted-foreground/50")}>
                                    {skin.images?.avatar && <Image src={skin.images.avatar || "/placeholder.svg"} alt={skin.displaySkin.skinName ?? "Skin"} fill className="object-cover" unoptimized />}
                                </button>
                            ))}
                        </div>
                    </div>
                </div>
            </InView>

            {/* Chibi viewer placeholder */}
            <InView
                variants={{
                    hidden: { opacity: 0, y: 20 },
                    visible: { opacity: 1, y: 0 },
                }}
                transition={{ duration: 0.4, delay: 0.1 }}
            >
                <ChibiViewer operator={operator} />
            </InView>

            {/* Fullscreen dialog */}
            <Dialog open={isFullscreen} onOpenChange={setIsFullscreen}>
                <DialogContent className="max-h-[90vh] max-w-[90vw] overflow-hidden p-0">
                    <DialogClose className="absolute right-4 top-4 z-10 rounded-full bg-background/80 p-2">
                        <X className="h-4 w-4" />
                    </DialogClose>
                    {selectedSkin?.images?.skin && (
                        <div className="relative h-[85vh] w-full">
                            <Image src={selectedSkin.images.skin || "/placeholder.svg"} alt={selectedSkin.displaySkin.skinName ?? "Skin"} fill className="object-contain" unoptimized />
                        </div>
                    )}
                </DialogContent>
            </Dialog>
        </div>
    );
}
