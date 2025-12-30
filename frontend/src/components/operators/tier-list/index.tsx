"use client";

import { BookOpen, ClipboardList, History } from "lucide-react";
import { useState } from "react";
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "~/components/ui/shadcn/accordion";
import { Badge } from "~/components/ui/shadcn/badge";
import { Card } from "~/components/ui/shadcn/card";
import type { TierListResponse, TierListVersionSummary } from "~/types/api/impl/tier-list";
import type { OperatorFromList } from "~/types/api/operators";
import { TierRow } from "./impl/tier-row";

interface TierListViewProps {
    tierListData: TierListResponse;
    operatorsData: Record<string, OperatorFromList>;
    versions: TierListVersionSummary[];
}

export function TierListView({ tierListData, operatorsData, versions }: TierListViewProps) {
    const [hoveredOperator, setHoveredOperator] = useState<string | null>(null);
    const [isGrayscaleActive, setIsGrayscaleActive] = useState(false);

    const handleOperatorHover = (operatorId: string | null, isHovered: boolean) => {
        if (isHovered) {
            setHoveredOperator(operatorId);
            setIsGrayscaleActive(true);
        } else {
            setHoveredOperator(null);
            setIsGrayscaleActive(false);
        }
    };

    return (
        <div className="min-w-0 space-y-6">
            {/* Header */}
            <div className="space-y-2">
                <h1 className="font-bold text-3xl text-foreground md:text-4xl">{tierListData.tier_list.name}</h1>
                {tierListData.tier_list.description && <p className="text-muted-foreground">{tierListData.tier_list.description}</p>}
            </div>

            {/* Info Accordion */}
            <Card className="border-border/50 bg-card/50 py-0">
                <Accordion className="px-4" collapsible type="single">
                    <AccordionItem value="how-it-works">
                        <AccordionTrigger>
                            <div className="flex items-center gap-2">
                                <BookOpen className="h-4 w-4 text-primary" />
                                <span>How This Tier List Works</span>
                            </div>
                        </AccordionTrigger>
                        <AccordionContent>
                            <div className="space-y-3 text-muted-foreground">
                                <p>
                                    Operators are ranked based on their overall performance, versatility, and impact across various game modes. Each tier represents a general power level:
                                </p>
                                <ul className="list-inside list-disc space-y-1 pl-2">
                                    <li><span className="font-medium text-foreground">S+ / S Tier:</span> Meta-defining operators that excel in most situations</li>
                                    <li><span className="font-medium text-foreground">A+ / A Tier:</span> Strong operators with high impact and good versatility</li>
                                    <li><span className="font-medium text-foreground">B+ / B Tier:</span> Solid choices that perform well in their niche</li>
                                    <li><span className="font-medium text-foreground">C / D Tier:</span> Situational operators or those outclassed by alternatives</li>
                                </ul>
                                <p className="text-sm italic">
                                    Note: Rankings may vary based on team composition, module upgrades, and specific stage requirements. Operators within the same tier are ordered left-to-right by general recommendation.
                                </p>
                            </div>
                        </AccordionContent>
                    </AccordionItem>

                    <AccordionItem value="criteria">
                        <AccordionTrigger>
                            <div className="flex items-center gap-2">
                                <ClipboardList className="h-4 w-4 text-primary" />
                                <span>Ranking Criteria</span>
                            </div>
                        </AccordionTrigger>
                        <AccordionContent>
                            <div className="space-y-3 text-muted-foreground">
                                <p>Operators are evaluated based on the following criteria:</p>
                                <div className="grid gap-3 sm:grid-cols-2">
                                    <div className="rounded-md border border-border/50 bg-muted/30 p-3">
                                        <p className="font-medium text-foreground text-sm">Combat Performance</p>
                                        <p className="text-xs">DPS output, survivability, crowd control effectiveness, and skill cycling</p>
                                    </div>
                                    <div className="rounded-md border border-border/50 bg-muted/30 p-3">
                                        <p className="font-medium text-foreground text-sm">Versatility</p>
                                        <p className="text-xs">Effectiveness across different stage types, enemy compositions, and team setups</p>
                                    </div>
                                    <div className="rounded-md border border-border/50 bg-muted/30 p-3">
                                        <p className="font-medium text-foreground text-sm">Ease of Use</p>
                                        <p className="text-xs">Deployment cost, skill timing requirements, and positioning flexibility</p>
                                    </div>
                                    <div className="rounded-md border border-border/50 bg-muted/30 p-3">
                                        <p className="font-medium text-foreground text-sm">Investment Value</p>
                                        <p className="text-xs">Performance relative to resource investment and availability of alternatives</p>
                                    </div>
                                </div>
                            </div>
                        </AccordionContent>
                    </AccordionItem>

                    <AccordionItem className="border-b-0" value="changelog">
                        <AccordionTrigger>
                            <div className="flex items-center gap-2">
                                <History className="h-4 w-4 text-primary" />
                                <span>Changelog</span>
                                {versions.length > 0 && (
                                    <Badge className="ml-1" variant="secondary">
                                        v{versions[0]?.version}
                                    </Badge>
                                )}
                            </div>
                        </AccordionTrigger>
                        <AccordionContent>
                            <div className="space-y-4 text-muted-foreground">
                                {versions.length > 0 ? (
                                    versions.map((version) => (
                                        <div className="border-primary/30 border-l-2 pl-3" key={version.id}>
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
                                            {version.change_summary && <p className="mt-1 text-sm">{version.change_summary}</p>}
                                        </div>
                                    ))
                                ) : (
                                    <>
                                        <div className="border-primary/30 border-l-2 pl-3">
                                            <p className="font-medium text-foreground text-sm">
                                                {new Date(tierListData.tier_list.updated_at).toLocaleDateString("en-US", {
                                                    year: "numeric",
                                                    month: "long",
                                                    day: "numeric",
                                                })}
                                            </p>
                                            <p className="text-sm">Initial tier list release</p>
                                        </div>
                                        <p className="text-muted-foreground/70 text-xs italic">
                                            Future updates will be documented here with detailed change notes.
                                        </p>
                                    </>
                                )}
                            </div>
                        </AccordionContent>
                    </AccordionItem>
                </Accordion>
            </Card>

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

            {/* Last Updated */}
            <div className="flex items-center justify-center pt-4 text-muted-foreground text-sm">
                <p>
                    Last updated:{" "}
                    {new Date(tierListData.tier_list.updated_at).toLocaleDateString("en-US", {
                        year: "numeric",
                        month: "long",
                        day: "numeric",
                    })}
                </p>
            </div>
        </div>
    );
}
