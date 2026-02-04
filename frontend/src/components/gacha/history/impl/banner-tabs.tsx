"use client";

import type React from "react";

import { Skeleton } from "~/components/ui/shadcn/skeleton";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/shadcn/tabs";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "~/components/ui/shadcn/tooltip";
import type { GachaRecords } from "~/types/api";

interface BannerTabsProps {
    records: GachaRecords;
    activeTab: string;
    onTabChange: (value: string) => void;
    children: React.ReactNode;
    isLoading?: boolean;
}

export function BannerTabs({ records: _records, activeTab, onTabChange, children, isLoading }: BannerTabsProps) {
    return (
        <Tabs className="w-full" onValueChange={onTabChange} value={activeTab}>
            <TabsList className="grid w-full grid-cols-4">
                <TabsTrigger value="all">All Banners</TabsTrigger>
                <TabsTrigger value="limited">Limited</TabsTrigger>
                <TabsTrigger value="regular">Regular</TabsTrigger>
                <TooltipProvider>
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <TabsTrigger value="special">Special</TabsTrigger>
                        </TooltipTrigger>
                        <TooltipContent>
                            <p>Includes Single-Pull and Collab banners</p>
                        </TooltipContent>
                    </Tooltip>
                </TooltipProvider>
            </TabsList>
            <TabsContent className="mt-6" value={activeTab}>
                {isLoading ? (
                    <div className="space-y-4">
                        <Skeleton className="h-32 w-full" />
                        <div className="grid gap-6 lg:grid-cols-[1fr_300px]">
                            <div className="space-y-1">
                                {Array.from({ length: 10 }).map((_, i) => (
                                    // biome-ignore lint/suspicious/noArrayIndexKey: Static skeleton placeholders
                                    <Skeleton className="h-16 w-full" key={i} />
                                ))}
                            </div>
                            <Skeleton className="h-96 w-full" />
                        </div>
                    </div>
                ) : (
                    children
                )}
            </TabsContent>
        </Tabs>
    );
}
