import { List, Loader2, User } from "lucide-react";
import { useState } from "react";
import { Alert, AlertDescription, AlertTitle } from "~/components/ui/alert";
import { Button } from "~/components/ui/button";
import { rarityToNumber } from "~/helper";
import type { Operator } from "~/types/impl/api/static/operator";
import type { OperatorOutcome, SelectedOpInfo, Tag } from "~/types/impl/frontend/impl/recruitment-calculator";
import { getRarityBorderColor, getRarityColor, PROFESSION_TO_TAG_ID } from "./helper";
import { POSITION_TO_TAG_ID } from "./helper";
import { RARITY_TO_TAG_ID } from "./helper";
import { cn } from "~/lib/utils";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "~/components/ui/hover-card";
import Image from "next/image";
import { Badge } from "~/components/ui/badge";

export const RecruitOutcome = ({
    allTagsMap,
    selectedOpInfo,
    setSelectedOpInfo,
    possibleOutcomes,
    isLoadingOutcomes,
    errorOutcomes,
    selectedTags,
}: {
    allTagsMap: Record<string, Tag>;
    selectedOpInfo: SelectedOpInfo | null;
    setSelectedOpInfo: (selectedOpInfo: SelectedOpInfo | null) => void;
    possibleOutcomes: OperatorOutcome[];
    isLoadingOutcomes: boolean;
    errorOutcomes: string | null;
    selectedTags: Set<string>;
}) => {
    const [viewMode, setViewMode] = useState<"text" | "profile">("text");

    const getOperatorDisplayTags = (op: Operator): string[] => {
        const tagsSet = new Set<string>();

        // Rarity Tag
        const rarityTagId = RARITY_TO_TAG_ID[rarityToNumber(op.rarity) - 1];
        if (rarityTagId && allTagsMap[rarityTagId]) {
            tagsSet.add(allTagsMap[rarityTagId].tagName);
        }

        // Position Tag
        const positionTagId = POSITION_TO_TAG_ID[op.position];
        if (positionTagId && allTagsMap[positionTagId]) {
            tagsSet.add(allTagsMap[positionTagId].tagName);
        }

        // Profession Tag
        const professionTagId = PROFESSION_TO_TAG_ID[op.profession];
        if (professionTagId && allTagsMap[professionTagId]) {
            tagsSet.add(allTagsMap[professionTagId].tagName);
        }

        // Affix Tags (assuming op.tagList exists and contains tag names)
        op.tagList?.forEach((tagName) => {
            // We could look up the tag object from allTagsMap if needed,
            // but just adding the name is sufficient here as per old code.
            tagsSet.add(tagName);
        });

        return Array.from(tagsSet);
    };

    const handleOperatorClick = (op: Operator, outcomeIndex: number) => {
        // If profile view is active, do nothing on click (handled by hover)
        if (viewMode === "profile") return;

        const opId = op.id ?? op.name; // Use ID or fallback to name
        // Check if clicking the already selected operator
        if (selectedOpInfo?.outcomeIndex === outcomeIndex && selectedOpInfo?.operatorId === opId) {
            setSelectedOpInfo(null); // Hide tags
        } else {
            // Calculate and store the tags for the newly clicked operator
            const allTags = getOperatorDisplayTags(op); // Reuse the tag calculation
            setSelectedOpInfo({
                outcomeIndex,
                operatorId: opId,
                allTags, // Store pre-calculated tags
            });
        }
    };

    return (
        <>
            <div>
                {/* header */}
                <div className="mb-2 flex items-center justify-between">
                    <h3 className="text-lg font-semibold">Possible Operator Outcomes</h3>
                </div>
                <div className="relative min-h-[150px] rounded-md border bg-muted/50 p-4">
                    {/* New container for Legend and Buttons */}
                    <div className="mb-2 flex min-h-[28px] items-center justify-between">
                        {" "}
                        {/* Added min-h for alignment when legend is hidden */}
                        {/* Legend for Profile View (Moved Here) */}
                        <div>
                            {" "}
                            {/* Left side container */}
                            {possibleOutcomes.length > 0 && (
                                <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
                                    {" "}
                                    {/* Legend content */}
                                    <span className="relative flex h-3 w-3 items-center justify-center rounded-full bg-blue-600 shadow-sm">
                                        <span className="absolute inline-flex h-full w-full animate-pulse rounded-full bg-blue-400 opacity-75"></span>
                                        <span className="relative inline-flex h-2 w-2 rounded-full bg-blue-500"></span>
                                    </span>
                                    <span>= Recruitment Only</span>
                                </div>
                            )}
                        </div>
                        {/* View Mode Toggle Buttons (Moved Here) */}
                        <div className="flex items-center gap-1">
                            <Button variant={viewMode === "text" ? "secondary" : "ghost"} size="sm" onClick={() => setViewMode("text")} title="Text View">
                                <List className="h-4 w-4" />
                            </Button>
                            <Button variant={viewMode === "profile" ? "secondary" : "ghost"} size="sm" onClick={() => setViewMode("profile")} title="Profile View">
                                <User className="h-4 w-4" />
                            </Button>
                        </div>
                    </div>

                    {isLoadingOutcomes && (
                        <div className="absolute inset-0 z-10 flex items-center justify-center rounded-md bg-background/80">
                            <Loader2 className="h-6 w-6 animate-spin text-primary" />
                            <span className="ml-2">Calculating...</span>
                        </div>
                    )}
                    {errorOutcomes && (
                        <Alert variant="destructive" className="mb-4">
                            <AlertTitle>Calculation Error</AlertTitle>
                            <AlertDescription>{errorOutcomes}</AlertDescription>
                        </Alert>
                    )}
                    {!isLoadingOutcomes && !errorOutcomes && (
                        <>
                            {selectedTags.size === 0 ? (
                                <p className="text-muted-foreground">Select tags above...</p>
                            ) : possibleOutcomes.length > 0 ? (
                                <div className="-mx-4 text-sm">
                                    {" "}
                                    {/* Container replacing table/tbody, negative margin to counter padding */}
                                    {possibleOutcomes.map(
                                        (outcome, outcomeIndex) =>
                                            outcome.operators &&
                                            outcome.operators.length > 0 && (
                                                <div key={outcomeIndex} className="flex flex-col border-b px-4 py-2 last:border-b-0 hover:bg-muted/20 md:flex-row">
                                                    {/* Tags Section (Combination) */}
                                                    <div className="md:w-1/3 md:pr-2">
                                                        <p className="mb-1 font-medium text-muted-foreground md:hidden">Combination:</p>
                                                        <div className="flex flex-wrap gap-1 pb-2 md:pb-0">
                                                            {outcome.label.map((tagName) => (
                                                                <div key={tagName} className="whitespace-nowrap rounded bg-secondary px-1.5 py-0.5 text-xs font-medium text-secondary-foreground">
                                                                    {tagName}
                                                                </div>
                                                            ))}
                                                        </div>
                                                    </div>

                                                    {/* Separator for mobile view */}
                                                    <div className="my-2 border-t md:hidden"></div>

                                                    {/* Operators Section */}
                                                    <div className="md:w-2/3 md:pl-2">
                                                        <p className="mb-1 font-medium text-muted-foreground md:hidden">Operators:</p>
                                                        {viewMode === "text" ? (
                                                            <div className="flex flex-wrap gap-x-2 gap-y-1">
                                                                {/* Text View Mapping */}
                                                                {outcome.operators
                                                                    .sort((a, b) => {
                                                                        const rarityA = rarityToNumber(a.rarity);
                                                                        const rarityB = rarityToNumber(b.rarity);
                                                                        if (rarityB !== rarityA) return rarityB - rarityA;
                                                                        return a.name.localeCompare(b.name);
                                                                    })
                                                                    .map((op) => {
                                                                        const rarityNum = rarityToNumber(op.rarity);
                                                                        const opId = op.id ?? op.name;
                                                                        const isSelected = selectedOpInfo?.outcomeIndex === outcomeIndex && selectedOpInfo?.operatorId === opId;
                                                                        return (
                                                                            <span
                                                                                key={opId}
                                                                                onClick={() => handleOperatorClick(op, outcomeIndex)}
                                                                                className={cn(getRarityColor(rarityNum), "cursor-pointer rounded px-1 py-0.5 hover:bg-primary/10", isSelected && "bg-primary/15 ring-1 ring-primary/25 transition-all duration-150")}
                                                                                title="Click to see all tags"
                                                                            >
                                                                                {op.name} ({rarityNum}★)
                                                                                {op.recruitOnly && (
                                                                                    <span className="ml-1.5 inline-flex h-2 w-2 items-center justify-center rounded-full bg-blue-600 align-middle shadow-sm" title="Recruitment Only">
                                                                                        <span className="inline-flex h-full w-full animate-pulse rounded-full bg-blue-400 opacity-75"></span>
                                                                                        <span className="relative -ml-2 inline-flex h-1.5 w-1.5 rounded-full bg-blue-500"></span>
                                                                                    </span>
                                                                                )}
                                                                            </span>
                                                                        );
                                                                    })}
                                                            </div>
                                                        ) : (
                                                            <div className="flex flex-wrap gap-x-1.5 gap-y-1.5 md:gap-x-2 md:gap-y-2">
                                                                {/* Profile View Mapping */}
                                                                {outcome.operators
                                                                    .sort((a, b) => {
                                                                        const rarityA = rarityToNumber(a.rarity);
                                                                        const rarityB = rarityToNumber(b.rarity);
                                                                        if (rarityB !== rarityA) return rarityB - rarityA;
                                                                        return a.name.localeCompare(b.name);
                                                                    })
                                                                    .map((op) => {
                                                                        const rarityNum = rarityToNumber(op.rarity);
                                                                        const opId = op.id ?? op.name;
                                                                        const imageUrl = `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/portrait/${op.id}_1.png`;
                                                                        const displayTags = getOperatorDisplayTags(op);

                                                                        return (
                                                                            <HoverCard key={opId} openDelay={200} closeDelay={100}>
                                                                                <HoverCardTrigger asChild>
                                                                                    <div
                                                                                        className={cn(
                                                                                            "relative aspect-[3/4] h-28 cursor-pointer overflow-hidden rounded transition-all hover:scale-105 hover:shadow-md md:h-36",
                                                                                            getRarityBorderColor(rarityNum), // Apply rarity border and shadow
                                                                                        )}
                                                                                        title={`${op.name}`}
                                                                                        tabIndex={0}
                                                                                    >
                                                                                        <Image src={imageUrl} alt={op.name} fill sizes="(max-width: 767px) 14vw, (min-width: 768px) 11vw" className="object-cover" loading="lazy" unoptimized />
                                                                                        <div className="absolute inset-x-0 bottom-0 bg-black/75 p-1 pt-1.5 backdrop-blur-sm">
                                                                                            <div className="truncate text-xs font-semibold">{op.name}</div>
                                                                                            <div className="text-[10px] text-yellow-300">{Array(rarityNum).fill("★").join("")}</div>
                                                                                        </div>
                                                                                        {op.recruitOnly && (
                                                                                            <span className="absolute right-1 top-1 flex h-3 w-3 items-center justify-center rounded-full bg-blue-600 shadow-md" title="Recruitment Only">
                                                                                                <span className="absolute inline-flex h-full w-full animate-pulse rounded-full bg-blue-400 opacity-75"></span>
                                                                                                <span className="relative inline-flex h-2 w-2 rounded-full bg-blue-500"></span>
                                                                                            </span>
                                                                                        )}
                                                                                    </div>
                                                                                </HoverCardTrigger>
                                                                                <HoverCardContent side="top" className="w-auto max-w-[250px] p-2">
                                                                                    <div className="flex flex-col gap-1">
                                                                                        <p className="mb-1 text-sm font-semibold">{op.name}</p>
                                                                                        <div className="flex flex-wrap gap-1">
                                                                                            {displayTags.length > 0 ? (
                                                                                                displayTags.map((tagName) => (
                                                                                                    <Badge variant="secondary" key={tagName}>
                                                                                                        {tagName}
                                                                                                    </Badge>
                                                                                                ))
                                                                                            ) : (
                                                                                                <p className="text-xs text-muted-foreground">No specific tags found.</p>
                                                                                            )}
                                                                                        </div>
                                                                                    </div>
                                                                                </HoverCardContent>
                                                                            </HoverCard>
                                                                        );
                                                                    })}
                                                            </div>
                                                        )}
                                                        {/* Conditionally display tags for the selected operator (TEXT VIEW ONLY) */}
                                                        {viewMode === "text" && selectedOpInfo?.outcomeIndex === outcomeIndex && (
                                                            <div className="mt-2 flex flex-wrap gap-1 border-t pt-2">
                                                                {selectedOpInfo.allTags.map((tagName) => (
                                                                    <div key={tagName} className="whitespace-nowrap rounded bg-accent px-1.5 py-0.5 text-xs text-accent-foreground">
                                                                        {tagName}
                                                                    </div>
                                                                ))}
                                                            </div>
                                                        )}
                                                    </div>
                                                </div>
                                            ),
                                    )}
                                </div>
                            ) : (
                                <p className="text-muted-foreground">No guaranteed outcomes found for the selected tags.</p>
                            )}
                        </>
                    )}
                </div>
            </div>
        </>
    );
};
