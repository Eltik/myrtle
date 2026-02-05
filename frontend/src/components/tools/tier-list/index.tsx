"use client";

import { BookOpen, ChevronRight, ClipboardList, Flag, HelpCircle, History } from "lucide-react";
import { useCallback, useState } from "react";
import { toast } from "sonner";
import { ReportTierListDialog, TierListTypeBadge } from "~/components/tier-lists";
import { Badge } from "~/components/ui/shadcn/badge";
import { Button } from "~/components/ui/shadcn/button";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "~/components/ui/shadcn/dialog";
import { Popover, PopoverContent, PopoverTrigger } from "~/components/ui/shadcn/popover";
import { ScrollArea } from "~/components/ui/shadcn/scroll-area";
import { Separator } from "~/components/ui/shadcn/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/shadcn/tabs";
import { useAuth } from "~/hooks/use-auth";
import type { OperatorFromList } from "~/types/api";
import type { TierListResponse, TierListVersionSummary } from "~/types/api/impl/tier-list";
import { TierRow } from "./impl/tier-row";
import { VersionDetailDialog } from "./impl/version-detail-dialog";

interface TierListViewProps {
    tierListData: TierListResponse;
    operatorsData: Record<string, OperatorFromList>;
    versions: TierListVersionSummary[];
}

export function TierListView({ tierListData, operatorsData, versions }: TierListViewProps) {
    const { user } = useAuth();
    const [hoveredOperator, setHoveredOperator] = useState<string | null>(null);
    const [isGrayscaleActive, setIsGrayscaleActive] = useState(false);
    const [selectedVersion, setSelectedVersion] = useState<TierListVersionSummary | null>(null);
    const [versionDialogOpen, setVersionDialogOpen] = useState(false);
    const [changelogDialogOpen, setChangelogDialogOpen] = useState(false);
    const [reportDialogOpen, setReportDialogOpen] = useState(false);

    const handleVersionClick = (version: TierListVersionSummary) => {
        setSelectedVersion(version);
        setVersionDialogOpen(true);
    };

    const handleOperatorHover = (operatorId: string | null, isHovered: boolean) => {
        if (isHovered) {
            setHoveredOperator(operatorId);
            setIsGrayscaleActive(true);
        } else {
            setHoveredOperator(null);
            setIsGrayscaleActive(false);
        }
    };

    const handleReport = useCallback(
        async (data: { reason: string; description?: string }) => {
            const response = await fetch(`/api/tier-lists/${tierListData.tier_list.slug}/report`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(data),
            });

            const result = await response.json();

            if (!response.ok) {
                throw new Error(result.error || "Failed to submit report");
            }

            toast.success("Report submitted. Thank you for helping keep the community safe.");
        },
        [tierListData.tier_list.slug],
    );

    const latestVersion = versions[0];

    // Check if user can report this tier list
    const isCommunityTierList = tierListData.tier_list.tier_list_type === "community";
    const isOwner = user?.status.uid === tierListData.tier_list.created_by;
    const canReport = user && isCommunityTierList && !isOwner;

    return (
        <div className="min-w-0 space-y-6">
            {/* Header */}
            <div className="space-y-2">
                <div className="flex items-start justify-between gap-4">
                    <div className="space-y-1">
                        <div className="flex items-center gap-3">
                            <h1 className="font-bold text-3xl text-foreground md:text-4xl">{tierListData.tier_list.name}</h1>
                            <TierListTypeBadge type={tierListData.tier_list.tier_list_type} />
                        </div>
                        {tierListData.tier_list.description && <p className="text-muted-foreground text-sm">{tierListData.tier_list.description}</p>}
                    </div>

                    <div className="flex shrink-0 items-center gap-2">
                        {/* Report Button - only for community tier lists when logged in and not owner */}
                        {canReport && (
                            <Button onClick={() => setReportDialogOpen(true)} size="icon" variant="ghost">
                                <Flag className="h-5 w-5 text-muted-foreground" />
                                <span className="sr-only">Report tier list</span>
                            </Button>
                        )}

                        {/* Info Button */}
                        <Popover>
                            <PopoverTrigger asChild>
                                <Button className="shrink-0" size="icon" variant="ghost">
                                    <HelpCircle className="h-5 w-5 text-muted-foreground" />
                                    <span className="sr-only">Tier list information</span>
                                </Button>
                            </PopoverTrigger>
                            <PopoverContent align="end" className="w-80">
                                <Tabs defaultValue="how-it-works">
                                    <TabsList className="w-full">
                                        <TabsTrigger className="flex-1 text-xs" value="how-it-works">
                                            <BookOpen className="mr-1.5 h-3 w-3" />
                                            How It Works
                                        </TabsTrigger>
                                        <Separator className="h-4" orientation="vertical" />
                                        <TabsTrigger className="flex-1 text-xs" value="criteria">
                                            <ClipboardList className="mr-1.5 h-3 w-3" />
                                            Criteria
                                        </TabsTrigger>
                                    </TabsList>
                                    <Separator className="my-3" />
                                    <TabsContent className="mt-0 space-y-2 text-muted-foreground text-sm" value="how-it-works">
                                        <p>Operators are ranked by overall performance across game modes:</p>
                                        <ul className="space-y-1 text-xs">
                                            <li>
                                                <span className="font-medium text-foreground">S+ / S:</span> Meta-defining, excel in most situations
                                            </li>
                                            <li>
                                                <span className="font-medium text-foreground">A+ / A:</span> Strong impact and good versatility
                                            </li>
                                            <li>
                                                <span className="font-medium text-foreground">B+ / B:</span> Solid choices in their niche
                                            </li>
                                            <li>
                                                <span className="font-medium text-foreground">C / D:</span> Situational or outclassed
                                            </li>
                                        </ul>
                                        <p className="text-muted-foreground/70 text-xs italic">Rankings vary by team comp and stage. Left-to-right = general recommendation.</p>
                                    </TabsContent>
                                    <TabsContent className="mt-0 space-y-2 text-muted-foreground text-sm" value="criteria">
                                        <p className="text-xs">Operators are evaluated on:</p>
                                        <div className="space-y-1.5">
                                            <div className="rounded border border-border/50 bg-muted/30 p-2">
                                                <p className="font-medium text-foreground text-xs">Combat Performance</p>
                                                <p className="text-2.5">DPS, survivability, CC, skill cycling</p>
                                            </div>
                                            <div className="rounded border border-border/50 bg-muted/30 p-2">
                                                <p className="font-medium text-foreground text-xs">Versatility</p>
                                                <p className="text-2.5">Effectiveness across stages and teams</p>
                                            </div>
                                            <div className="rounded border border-border/50 bg-muted/30 p-2">
                                                <p className="font-medium text-foreground text-xs">Ease of Use</p>
                                                <p className="text-2.5">Cost, timing, positioning flexibility</p>
                                            </div>
                                            <div className="rounded border border-border/50 bg-muted/30 p-2">
                                                <p className="font-medium text-foreground text-xs">Investment Value</p>
                                                <p className="text-2.5">Performance vs resources spent</p>
                                            </div>
                                        </div>
                                    </TabsContent>
                                </Tabs>
                            </PopoverContent>
                        </Popover>
                    </div>
                </div>

                {/* Version Info - Below title */}
                <div className="flex flex-wrap items-center gap-x-3 gap-y-1 text-muted-foreground text-sm">
                    <span>
                        Updated{" "}
                        {new Date(tierListData.tier_list.updated_at).toLocaleDateString("en-US", {
                            year: "numeric",
                            month: "short",
                            day: "numeric",
                        })}
                    </span>
                    {versions.length > 0 && (
                        <>
                            <span className="text-muted-foreground/50">|</span>
                            <button className="inline-flex items-center gap-1.5 text-primary hover:underline" onClick={() => setChangelogDialogOpen(true)} type="button">
                                <History className="h-3.5 w-3.5" />
                                <span>v{latestVersion?.version}</span>
                            </button>
                        </>
                    )}
                </div>
            </div>

            {/* Tier List */}
            <div className="space-y-4">
                {tierListData.tiers.map((tier) => {
                    const operators = tier.placements
                        .sort((a, b) => a.sub_order - b.sub_order)
                        .map((placement) => operatorsData[placement.operator_id])
                        .filter((op): op is OperatorFromList => op !== undefined);

                    return <TierRow hoveredOperator={hoveredOperator} isGrayscaleActive={isGrayscaleActive} key={tier.id} onOperatorHover={handleOperatorHover} operators={operators} tier={tier} />;
                })}
            </div>

            {/* Changelog Dialog */}
            <Dialog onOpenChange={setChangelogDialogOpen} open={changelogDialogOpen}>
                <DialogContent className="max-w-lg">
                    <DialogHeader>
                        <DialogTitle className="flex items-center gap-2">
                            <History className="h-5 w-5" />
                            Changelog
                        </DialogTitle>
                        <DialogDescription>Version history and updates for this tier list</DialogDescription>
                    </DialogHeader>
                    <ScrollArea className="max-h-[60vh]">
                        <div className="space-y-2 pr-4">
                            {versions.length > 0 ? (
                                versions.map((version) => (
                                    <button
                                        className="group w-full rounded-md border border-border p-3 text-left transition-colors hover:border-border hover:bg-muted/50"
                                        key={version.id}
                                        onClick={() => {
                                            setChangelogDialogOpen(false);
                                            handleVersionClick(version);
                                        }}
                                        type="button"
                                    >
                                        <div className="flex items-center justify-between gap-2">
                                            <div className="flex items-center gap-2">
                                                <p className="font-medium text-foreground text-sm">
                                                    {new Date(version.published_at).toLocaleDateString("en-US", {
                                                        year: "numeric",
                                                        month: "long",
                                                        day: "numeric",
                                                    })}
                                                </p>
                                                <Badge variant="outline">v{version.version}</Badge>
                                            </div>
                                            <ChevronRight className="h-4 w-4 text-muted-foreground opacity-0 transition-opacity group-hover:opacity-100" />
                                        </div>
                                        {version.change_summary && <p className="mt-1 text-muted-foreground text-sm">{version.change_summary}</p>}
                                    </button>
                                ))
                            ) : (
                                <div className="py-8 text-center text-muted-foreground">
                                    <p>No versions published yet.</p>
                                </div>
                            )}
                        </div>
                    </ScrollArea>
                </DialogContent>
            </Dialog>

            {/* Version Detail Dialog */}
            <VersionDetailDialog onOpenChange={setVersionDialogOpen} open={versionDialogOpen} tierListSlug={tierListData.tier_list.slug} version={selectedVersion} />

            {/* Report Dialog */}
            <ReportTierListDialog onOpenChange={setReportDialogOpen} onReport={handleReport} open={reportDialogOpen} tierListName={tierListData.tier_list.name} tierListSlug={tierListData.tier_list.slug} />
        </div>
    );
}
