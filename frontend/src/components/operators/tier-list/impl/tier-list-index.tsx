"use client";

import { ArrowRight, Calendar, LayoutList, Users } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import Image from "next/image";
import Link from "next/link";
import { useState } from "react";
import { TierListTypeBadge } from "~/components/tier-lists/tier-list-type-badge";
import { Badge } from "~/components/ui/shadcn/badge";
import { Tabs, TabsList, TabsTrigger } from "~/components/ui/shadcn/tabs";
import { cn } from "~/lib/utils";
import type { TierListType } from "~/types/api/impl/tier-list";
import type { OperatorFromList } from "~/types/api/operators";

interface TierListPreview {
    id: string;
    name: string;
    slug: string;
    description: string | null;
    is_active: boolean;
    tier_list_type: TierListType;
    created_at: string;
    updated_at: string;
    operatorCount: number;
    tierCount: number;
    topOperators: OperatorFromList[];
}

interface TierListIndexProps {
    tierLists: TierListPreview[];
}

type FilterType = "all" | "official" | "community";

const CONTAINER_TRANSITION = {
    type: "spring" as const,
    stiffness: 500,
    damping: 30,
};

export function TierListIndex({ tierLists }: TierListIndexProps) {
    const [filter, setFilter] = useState<FilterType>("all");

    const activeTierLists = tierLists.filter((tl) => tl.is_active);
    const filteredTierLists = activeTierLists.filter((tl) => {
        if (filter === "all") return true;
        return tl.tier_list_type === filter;
    });

    const officialCount = activeTierLists.filter((tl) => tl.tier_list_type === "official").length;
    const communityCount = activeTierLists.filter((tl) => tl.tier_list_type === "community").length;

    return (
        <div className="min-w-0 space-y-6">
            {/* Header */}
            <div className="space-y-2">
                <h1 className="font-bold text-3xl text-foreground md:text-4xl">Tier Lists</h1>
                <p className="text-muted-foreground">Browse operator tier lists and rankings</p>
            </div>

            {/* Filter Tabs */}
            <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                <Tabs defaultValue="all" onValueChange={(v) => setFilter(v as FilterType)} value={filter}>
                    <TabsList>
                        <TabsTrigger value="all">All ({activeTierLists.length})</TabsTrigger>
                        <TabsTrigger value="official">Official ({officialCount})</TabsTrigger>
                        <TabsTrigger value="community">Community ({communityCount})</TabsTrigger>
                    </TabsList>
                </Tabs>
                <div className="flex items-center gap-1.5 text-muted-foreground text-sm">
                    <LayoutList className="h-4 w-4" />
                    <span>{filteredTierLists.length} tier lists</span>
                </div>
            </div>

            {/* Tier Lists Grid */}
            <AnimatePresence mode="wait">
                {filteredTierLists.length > 0 ? (
                    <motion.div animate={{ opacity: 1, y: 0 }} className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3" exit={{ opacity: 0, y: 8 }} initial={{ opacity: 0, y: 8 }} key={filter} transition={CONTAINER_TRANSITION}>
                        {filteredTierLists.map((tierList, index) => (
                            <motion.div
                                animate={{ opacity: 1, y: 0 }}
                                initial={{ opacity: 0, y: 8 }}
                                key={tierList.id}
                                transition={{
                                    duration: 0.2,
                                    delay: Math.min(index * 0.05, 0.2),
                                    ease: [0.4, 0, 0.2, 1],
                                }}
                            >
                                <TierListCard tierList={tierList} />
                            </motion.div>
                        ))}
                    </motion.div>
                ) : (
                    <motion.div animate={{ opacity: 1, scale: 1 }} className="flex flex-col items-center justify-center py-16 text-center" exit={{ opacity: 0, scale: 0.98 }} initial={{ opacity: 0, scale: 0.98 }} key={`empty-${filter}`} transition={CONTAINER_TRANSITION}>
                        <div className="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-secondary">
                            <LayoutList className="h-8 w-8 text-muted-foreground" />
                        </div>
                        <h3 className="mb-2 font-semibold text-foreground text-lg">No tier lists available</h3>
                        <p className="max-w-sm text-muted-foreground text-sm">{filter === "community" ? "No community tier lists have been created yet." : filter === "official" ? "No official tier lists are available." : "Check back later for operator tier lists and rankings."}</p>
                    </motion.div>
                )}
            </AnimatePresence>
        </div>
    );
}

function TierListCard({ tierList }: { tierList: TierListPreview }) {
    const formattedDate = new Date(tierList.updated_at).toLocaleDateString("en-US", {
        year: "numeric",
        month: "short",
        day: "numeric",
    });

    return (
        <Link className="group relative flex flex-col overflow-hidden rounded-xl border border-border bg-card/50 transition-all duration-200 hover:border-primary/50 hover:bg-card" href={`/operators/tier-list?slug=${tierList.slug}`}>
            {/* Preview Header - Operator portraits */}
            <div className="relative h-28 overflow-hidden bg-gradient-to-b from-muted/50 to-transparent">
                {tierList.topOperators.length > 0 ? (
                    <div className="flex h-full w-full items-center justify-center gap-1 overflow-hidden px-2 py-3">
                        {tierList.topOperators.slice(0, 5).map((operator, idx) => (
                            <div
                                className={cn("relative aspect-3/4 shrink-0 overflow-hidden rounded-md border border-muted/50 bg-card transition-transform duration-200", idx < 3 ? "h-18 group-hover:scale-105 sm:h-20" : "hidden h-16 opacity-70 group-hover:opacity-90 sm:block")}
                                key={operator.id}
                                style={{
                                    zIndex: 5 - idx,
                                }}
                            >
                                <Image alt={operator.name ?? "Operator"} className="h-full w-full object-cover object-top" fill loading="lazy" src={`/api/cdn${operator.portrait}`} />
                            </div>
                        ))}
                    </div>
                ) : (
                    <div className="flex h-full items-center justify-center">
                        <LayoutList className="h-10 w-10 text-muted-foreground/30" />
                    </div>
                )}

                {/* Gradient overlay */}
                <div className="absolute inset-x-0 bottom-0 h-12 bg-gradient-to-t from-card/50 to-transparent" />
            </div>

            {/* Content */}
            <div className="flex flex-1 flex-col gap-3 p-4">
                {/* Title and Description */}
                <div className="space-y-1.5">
                    <div className="flex items-center justify-between gap-2">
                        <h3 className="font-semibold text-foreground text-lg transition-colors group-hover:text-primary">{tierList.name}</h3>
                        <ArrowRight className="h-4 w-4 shrink-0 text-muted-foreground opacity-0 transition-all duration-200 group-hover:translate-x-0.5 group-hover:text-primary group-hover:opacity-100" />
                    </div>
                    {tierList.description && <p className="line-clamp-2 text-muted-foreground text-sm">{tierList.description}</p>}
                </div>

                {/* Stats and Meta */}
                <div className="mt-auto flex flex-wrap items-center gap-2 pt-2">
                    <TierListTypeBadge showIcon={false} type={tierList.tier_list_type} />
                    <Badge className="gap-1 bg-secondary/50 text-muted-foreground" variant="secondary">
                        <Users className="h-3 w-3" />
                        {tierList.operatorCount} operators
                    </Badge>
                    <Badge className="gap-1 bg-secondary/50 text-muted-foreground" variant="secondary">
                        <LayoutList className="h-3 w-3" />
                        {tierList.tierCount} tiers
                    </Badge>
                </div>

                {/* Footer */}
                <div className="flex items-center gap-1.5 border-border/50 border-t pt-3 text-muted-foreground text-xs">
                    <Calendar className="h-3 w-3" />
                    <span>Updated {formattedDate}</span>
                </div>
            </div>
        </Link>
    );
}
