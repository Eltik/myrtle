"use client";

import { ChevronDown, ChevronUp, Hammer, Layers, MapPin, Tag } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useState } from "react";
import { Badge } from "~/components/ui/shadcn/badge";
import { Button } from "~/components/ui/shadcn/button";
import { ScrollArea } from "~/components/ui/shadcn/scroll-area";
import { Separator } from "~/components/ui/shadcn/separator";
import { OCC_PER_LABELS, RARITY_COLORS, RARITY_GLOW, RARITY_LABELS } from "./constants";
import { formatClassType, formatItemType } from "./helpers";
import { ItemIconLarge } from "./item-icon";
import type { ItemWithData } from "./types";

interface ItemDetailCardProps {
    item: ItemWithData;
}

export function ItemDetailCard({ item }: ItemDetailCardProps) {
    const [showDetails, setShowDetails] = useState(false);

    const defaultColors = { bg: "bg-neutral-100 dark:bg-neutral-800", text: "text-neutral-600 dark:text-neutral-300", border: "border-neutral-200 dark:border-neutral-700" };
    const rarityColors = RARITY_COLORS[item.rarity ?? "TIER_1"] ?? defaultColors;
    const rarityLabel = RARITY_LABELS[item.rarity ?? "TIER_1"] ?? "Unknown";
    const imageUrl = item.image ? `/api/cdn${item.image}` : `/api/cdn/upk/spritepack/ui_item_icons_h1_0/${item.iconId ?? item.id}.png`;

    return (
        <div className="w-full max-w-lg">
            {/* Header with rarity-colored background */}
            <div className={`rounded-t-xl p-4 ${rarityColors.bg}`}>
                <div className="flex items-start gap-4">
                    {/* Item Icon */}
                    <div className={`shrink-0 rounded-xl bg-background/80 p-2 shadow-sm ${RARITY_GLOW[item.rarity ?? "TIER_1"] ?? ""}`}>
                        <ItemIconLarge alt={item.name ?? item.id} src={imageUrl} />
                    </div>

                    {/* Item Info */}
                    <div className="min-w-0 flex-1">
                        <h2 className={`font-bold text-xl ${rarityColors.text}`}>{item.name ?? item.id}</h2>
                        <p className="mt-0.5 text-muted-foreground text-sm">{item.id}</p>
                        <div className="mt-2 flex items-center gap-2">
                            <Badge className="font-mono" variant="secondary">
                                <Layers className="mr-1 h-3 w-3" />
                                {item.displayAmount.toLocaleString()}
                            </Badge>
                            <Badge className={rarityColors.border} variant="outline">
                                {rarityLabel}
                            </Badge>
                        </div>
                    </div>
                </div>
            </div>

            {/* Content */}
            <div className="space-y-4 rounded-b-xl border border-t-0 bg-background p-4">
                {/* Description */}
                {item.description && (
                    <div>
                        <p className="text-muted-foreground text-sm leading-relaxed">{item.description}</p>
                    </div>
                )}

                {/* Type Badges */}
                <div className="flex flex-wrap gap-2">
                    {item.classifyType && (
                        <Badge variant="secondary">
                            <Tag className="mr-1 h-3 w-3" />
                            {formatClassType(item.classifyType)}
                        </Badge>
                    )}
                    {item.itemType && (
                        <Badge variant="secondary">
                            <Tag className="mr-1 h-3 w-3" />
                            {formatItemType(item.itemType)}
                        </Badge>
                    )}
                </div>

                <Separator />

                {/* Usage */}
                {item.usage && (
                    <div>
                        <h3 className="mb-2 font-semibold text-sm">Usage</h3>
                        <p className="text-muted-foreground text-sm leading-relaxed">{item.usage}</p>
                    </div>
                )}

                {/* Building Production */}
                {item.buildingProductList && item.buildingProductList.length > 0 && (
                    <div>
                        <h3 className="mb-2 font-semibold text-sm">Building Production</h3>
                        <div className="space-y-1.5">
                            {item.buildingProductList.map((product) => (
                                <div className="flex items-center gap-2 rounded-md bg-muted/50 px-3 py-1.5 text-sm" key={product.formulaId}>
                                    <Hammer className="h-4 w-4 text-muted-foreground" />
                                    <span className="font-medium">{product.roomType}</span>
                                    <span className="text-muted-foreground">•</span>
                                    <span className="text-muted-foreground">{product.formulaId}</span>
                                </div>
                            ))}
                        </div>
                    </div>
                )}

                {/* Expandable Details Button */}
                {(item.obtainApproach || (item.stageDropList && item.stageDropList.length > 0)) && (
                    <>
                        <Button className="w-full" onClick={() => setShowDetails(!showDetails)} size="sm" variant="outline">
                            {showDetails ? (
                                <>
                                    <ChevronUp className="mr-2 h-4 w-4" />
                                    Hide Details
                                </>
                            ) : (
                                <>
                                    <ChevronDown className="mr-2 h-4 w-4" />
                                    Show Details
                                </>
                            )}
                        </Button>

                        {/* Expandable Content */}
                        <AnimatePresence>
                            {showDetails && (
                                <motion.div animate={{ height: "auto", opacity: 1 }} className="overflow-hidden" exit={{ height: 0, opacity: 0 }} initial={{ height: 0, opacity: 0 }} transition={{ duration: 0.2 }}>
                                    <div className="space-y-4 pt-2">
                                        {/* How to Obtain */}
                                        {item.obtainApproach && (
                                            <div>
                                                <h3 className="mb-2 font-semibold text-sm">How to Obtain</h3>
                                                <p className="text-muted-foreground text-sm">{item.obtainApproach}</p>
                                            </div>
                                        )}

                                        {/* Drop Stages */}
                                        {item.stageDropList && item.stageDropList.length > 0 && (
                                            <div>
                                                <h3 className="mb-2 font-semibold text-sm">Drop Stages</h3>
                                                <ScrollArea className="h-40 rounded-md border">
                                                    <div className="space-y-1.5 p-2">
                                                        {item.stageDropList.map((drop) => (
                                                            <div className="flex items-center justify-between rounded-md bg-muted/50 px-3 py-1.5 text-sm" key={drop.stageId}>
                                                                <span className="flex items-center gap-2">
                                                                    <MapPin className="h-4 w-4 text-muted-foreground" />
                                                                    {drop.stageId}
                                                                </span>
                                                                <Badge className="text-xs" variant="outline">
                                                                    {OCC_PER_LABELS[drop.occPer] ?? drop.occPer}
                                                                </Badge>
                                                            </div>
                                                        ))}
                                                    </div>
                                                </ScrollArea>
                                            </div>
                                        )}
                                    </div>
                                </motion.div>
                            )}
                        </AnimatePresence>
                    </>
                )}
            </div>
        </div>
    );
}
