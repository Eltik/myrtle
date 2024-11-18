"use client";

import React, { useState } from "react";
import { ChevronDown, ChevronUp, Hammer, MapPin, Package } from "lucide-react";

// Import the types from the provided type definitions
import { type Item, ItemRarity, ItemClass, ItemType, ItemOccPer, BuildingRoomType, VoucherItemType } from "~/types/impl/api/static/material";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "../ui/card";
import { Badge } from "../ui/badge";
import { Separator } from "../ui/separator";
import { ScrollArea } from "../ui/scroll-area";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "../ui/tooltip";
import { Button } from "../ui/button";

const rarityColors = {
    [ItemRarity.TIER_1]: "bg-gray-200 text-gray-800",
    [ItemRarity.TIER_2]: "bg-green-200 text-green-800",
    [ItemRarity.TIER_3]: "bg-blue-200 text-blue-800",
    [ItemRarity.TIER_4]: "bg-purple-200 text-purple-800",
    [ItemRarity.TIER_5]: "bg-yellow-200 text-yellow-800",
    [ItemRarity.TIER_6]: "bg-red-200 text-red-800",
};

const occPerLabels = {
    [ItemOccPer.USUAL]: "Usual",
    [ItemOccPer.ALMOST]: "Almost",
    [ItemOccPer.ALWAYS]: "Always",
    [ItemOccPer.SOMETIMES]: "Sometimes",
};

export default function ItemDialogueCard({ item }: { item: Item }) {
    const [showDetails, setShowDetails] = useState(false);

    return (
        <Card className="w-full max-w-2xl border-none bg-background p-0 shadow-lg">
            <CardHeader className={`${rarityColors[item.rarity]} rounded-t-lg`}>
                <div className="flex items-center space-x-4">
                    <img src={`https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/item/${item.iconId}.png`} alt={item.name} className="h-16 w-16 rounded-full bg-background p-1" />
                    <div>
                        <CardTitle className="text-2xl font-bold">{item.name}</CardTitle>
                        <CardDescription className="line-clamp-2 text-sm opacity-90 sm:line-clamp-none">{item.description}</CardDescription>
                    </div>
                </div>
            </CardHeader>
            <CardContent className="space-y-4 pt-6">
                <div className="flex flex-wrap gap-2">
                    <Badge variant="secondary">{ItemClass[item.classifyType as keyof typeof ItemClass]}</Badge>
                    <Badge variant="secondary">{ItemType[item.itemType]}</Badge>
                    <Badge variant="outline">Rarity: {item.rarity}</Badge>
                </div>
                <Separator />
                <ScrollArea className="h-64 sm:h-full">
                    <div>
                        <h3 className="mb-2 font-semibold">Usage</h3>
                        <p className="text-sm text-muted-foreground">{item.usage}</p>
                    </div>
                    <br />
                    {item.buildingProductList.length > 0 ? (
                        <div>
                            <h3 className="mb-2 font-semibold">Building Production</h3>
                            <ul className="space-y-2">
                                {item.buildingProductList.map((product) => (
                                    <li key={product.formulaId} className="flex items-center text-sm">
                                        <Hammer className="mr-2 h-4 w-4" />
                                        {BuildingRoomType[product.roomType]}: {product.formulaId}
                                    </li>
                                ))}
                            </ul>
                        </div>
                    ) : (
                        <div>
                            <h3 className="mb-2 font-semibold">Building Production</h3>
                            <p className="text-sm text-muted-foreground">This item is not used in any building production.</p>
                        </div>
                    )}
                </ScrollArea>
            </CardContent>
            <CardFooter>
                <TooltipProvider>
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <Button variant="outline" className="w-full" onClick={() => setShowDetails(!showDetails)}>
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
                        </TooltipTrigger>
                        <TooltipContent>
                            <p>Click to {showDetails ? "hide" : "show"} additional item details</p>
                        </TooltipContent>
                    </Tooltip>
                </TooltipProvider>
            </CardFooter>
            {showDetails && (
                <CardContent className="pt-0">
                    <Separator className="my-4" />
                    <ScrollArea className="max-h-32">
                        <div className="space-y-4">
                            {!item.obtainApproach && item.stageDropList.length === 0 && <p className="text-sm text-muted-foreground">No additional details available.</p>}
                            {item.obtainApproach && (
                                <div>
                                    <h3 className="mb-2 font-semibold">How to Obtain</h3>
                                    <p className="text-sm text-muted-foreground">{item.obtainApproach}</p>
                                </div>
                            )}
                            {item.stageDropList.length > 0 && (
                                <div>
                                    <h3 className="mb-2 font-semibold">Drop Stages</h3>
                                    <ScrollArea className="max-h-32 sm:h-24">
                                        <ul className="space-y-2">
                                            {item.stageDropList.map((drop) => (
                                                <li key={drop.stageId} className="flex items-center justify-between text-sm">
                                                    <span className="flex items-center">
                                                        <MapPin className="mr-2 h-4 w-4" />
                                                        {drop.stageId}
                                                    </span>
                                                    <Badge variant="outline">{occPerLabels[drop.occPer] ?? "None"}</Badge>
                                                </li>
                                            ))}
                                        </ul>
                                    </ScrollArea>
                                </div>
                            )}
                            {item.voucherRelateList && item.voucherRelateList.length > 0 && (
                                <div>
                                    <h3 className="mb-2 font-semibold">Related Vouchers</h3>
                                    <ul className="space-y-2">
                                        {item.voucherRelateList.map((voucher) => (
                                            <li key={voucher.voucherId} className="flex items-center text-sm">
                                                <Package className="mr-2 h-4 w-4" />
                                                {voucher.voucherId} ({VoucherItemType[voucher.voucherItemType]})
                                            </li>
                                        ))}
                                    </ul>
                                </div>
                            )}
                        </div>
                    </ScrollArea>
                </CardContent>
            )}
        </Card>
    );
}
