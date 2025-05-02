"use client";

import React, { useState, useEffect, useCallback } from "react";
import { Checkbox } from "~/components/ui/checkbox";
import { Card, CardContent } from "~/components/ui/card";
import { Button } from "~/components/ui/button";
import { Separator } from "~/components/ui/separator";
import { ScrollArea } from "~/components/ui/scroll-area";
import { Alert, AlertDescription, AlertTitle } from "~/components/ui/alert";
import { Loader2 } from "lucide-react";
import type { Operator } from "~/types/impl/api/static/operator";
import { rarityToNumber } from "~/helper";
import { cn } from "~/lib/utils";
import Link from "next/link";

// Define expected data structures
interface Tag {
    tagId: string;
    tagName: string;
    tagGroup: number;
    tagCat?: string;
}

// Updated OperatorOutcome structure
type OperatorOutcome = {
    label: string[];
    operators: (Operator & { recruitOnly: boolean })[];
};

// Use Record for better type safety
type GroupedTags = Record<string, Tag[]>;

// Define the expected structure of the API response (Success)
interface ApiResponse<T> {
    data: T;
    error?: string; // Should ideally not be present on success
}

// Define a potential structure for error responses from the API proxy
interface ErrorResponse {
    error?: string;
}

// Type for storing info about the clicked operator for tag display
interface SelectedOpInfo {
    outcomeIndex: number;
    operatorId: string;
    allTags: string[]; // Store pre-calculated tags here
}

// Helper map for operator profession to tag ID (based on old code's logic)
const PROFESSION_TO_TAG_ID: Record<string, string> = {
    Guard: "1",
    Sniper: "2",
    Defender: "3",
    Medic: "4",
    Supporter: "5",
    Caster: "6",
    Specialist: "7",
    Vanguard: "8",
};

// Helper map for position to tag ID
const POSITION_TO_TAG_ID: Record<string, string> = {
    MELEE: "9",
    RANGED: "10",
};

// Helper map for rarity to tag ID
const RARITY_TO_TAG_ID: Record<number, string> = {
    0: "28", // Robot
    1: "17", // Starter (1*)
    4: "14", // Senior Op
    5: "11", // Top Op
};

const RecruitmentCalculator = () => {
    const [allTagsMap, setAllTagsMap] = useState<Record<string, Tag>>({}); // For easy lookup
    const [groupedTags, setGroupedTags] = useState<GroupedTags>({});
    const [selectedTags, setSelectedTags] = useState<Set<string>>(new Set());
    // State is now an array again
    const [possibleOutcomes, setPossibleOutcomes] = useState<OperatorOutcome[]>([]);
    const [isLoadingTags, setIsLoadingTags] = useState<boolean>(true);
    const [isLoadingOutcomes, setIsLoadingOutcomes] = useState<boolean>(false);
    const [errorTags, setErrorTags] = useState<string | null>(null);
    const [errorOutcomes, setErrorOutcomes] = useState<string | null>(null);
    // State to manage showing tags for a clicked operator
    const [selectedOpInfo, setSelectedOpInfo] = useState<SelectedOpInfo | null>(null);

    // Fetch all tags from the backend via the /api/static proxy
    useEffect(() => {
        const fetchTags = async () => {
            setIsLoadingTags(true);
            setErrorTags(null);
            try {
                const response = await fetch("/api/static", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({ type: "gacha", method: "recruitment" }),
                });

                if (!response.ok) {
                    let errorData: ErrorResponse = {};
                    try {
                        errorData = (await response.json()) as ErrorResponse;
                    } catch {
                        console.warn("Could not parse error response body as JSON.");
                    }
                    throw new Error(`HTTP error ${response.status}: ${errorData?.error ?? response.statusText}`);
                }

                // Expect { data: Tag[] } on success
                const result = (await response.json()) as ApiResponse<Tag[]>;

                if (result.error) {
                    // Handle potential errors returned even with 2xx status
                    throw new Error(`API error: ${result.error}`);
                }
                if (!result.data) {
                    throw new Error("No tag data received from API.");
                }

                const tagsData = result.data;

                // Group tags by type for display
                const grouped = tagsData.reduce((acc, tag) => {
                    const type = tag.tagCat ? tag.tagCat : "Other";
                    if (!acc[type]) {
                        acc[type] = [];
                    }
                    acc[type].push(tag);
                    return acc;
                }, {} as GroupedTags);

                const tagsMap = tagsData.reduce(
                    (acc, tag) => {
                        acc[tag.tagId] = tag;
                        return acc;
                    },
                    {} as Record<string, Tag>,
                ); // Create lookup map

                setGroupedTags(grouped);
                setAllTagsMap(tagsMap); // Store the map
            } catch (error) {
                console.error("Failed to fetch tags:", error);
                setErrorTags(`Failed to load tags. ${error instanceof Error ? error.message : String(error)}`);
            } finally {
                setIsLoadingTags(false);
            }
        };
        void fetchTags();
    }, []);

    // Calculate outcomes by sending selected tags to the /api/static proxy
    const calculateOutcomes = useCallback(async (tagsToCalculate: Set<string>) => {
        if (tagsToCalculate.size === 0) {
            // Set state back to empty array
            setPossibleOutcomes([]);
            setErrorOutcomes(null);
            setIsLoadingOutcomes(false);
            setSelectedOpInfo(null); // Clear selected op tags when calculation runs
            return;
        }

        setIsLoadingOutcomes(true);
        setErrorOutcomes(null);
        try {
            const response = await fetch("/api/static", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    type: "gacha",
                    method: "calculate",
                    tags: Array.from(tagsToCalculate),
                }),
            });

            if (!response.ok) {
                let errorData: ErrorResponse = {};
                try {
                    errorData = (await response.json()) as ErrorResponse;
                } catch {
                    console.warn("Could not parse error response body as JSON.");
                }
                throw new Error(`HTTP error ${response.status}: ${errorData?.error ?? response.statusText}`);
            }

            // Expect { data: OperatorOutcome[] } (array) on success
            const result = (await response.json()) as ApiResponse<OperatorOutcome[]>;

            if (result.error) {
                throw new Error(`API error: ${result.error}`);
            }

            // Set state to the array or empty array if no data
            setPossibleOutcomes(result.data ?? []);
            if (!result.data) {
                console.log("Calculation returned no specific outcomes (or backend returned null/undefined data).");
            }

            // Clear selected op tags when calculation runs
            setSelectedOpInfo(null);
        } catch (error) {
            console.error("Failed to calculate outcomes:", error);
            setErrorOutcomes(`Calculation failed. ${error instanceof Error ? error.message : String(error)}`);
            // Ensure state is empty array on error
            setPossibleOutcomes([]);
        } finally {
            setIsLoadingOutcomes(false);
        }
    }, []);

    // Recalculate when selected tags change, handle potential promise rejection
    useEffect(() => {
        // Only proceed with calculation if tags are actually selected
        if (selectedTags.size === 0) {
            // Set state back to empty array
            setPossibleOutcomes([]);
            setErrorOutcomes(null);
            // No need to debounce or call API if empty
            return;
        }

        // Debounced calculation for non-empty selections
        const debouncedCalculate = () => {
            console.log("Debounced calculation triggered for tags:", selectedTags);
            void (async () => {
                try {
                    // calculateOutcomes already handles internal loading/error states
                    await calculateOutcomes(selectedTags);
                } catch (error) {
                    // Log any unexpected error from the async wrapper itself
                    console.error("Error during debounced calculation wrapper:", error);
                }
            })();
        };

        const timer = setTimeout(debouncedCalculate, 300); // Simple debounce

        // Cleanup function to clear the timeout
        return () => clearTimeout(timer);
    }, [selectedTags, calculateOutcomes]); // Dependencies remain the same

    // Add explicit type annotation for 'checked'
    const handleTagChange = (tagId: string, checked: boolean | "indeterminate") => {
        setSelectedTags((prev) => {
            const next = new Set(prev);
            if (checked === true) {
                // Explicitly check for true
                if (next.size < 5) {
                    // Limit to 5 selections
                    next.add(tagId);
                } else {
                    // Optional: Provide user feedback that max tags are selected
                    console.warn("Maximum of 5 tags already selected.");
                    return prev; // Prevent adding more than 5
                }
            } else {
                next.delete(tagId);
            }
            return next;
        });
    };

    const resetSelection = () => {
        setSelectedTags(new Set());
        setSelectedOpInfo(null); // Clear selected op tags on reset
    };

    // Helper function to get rarity color - adjust as needed
    const getRarityColor = (rarityNum: number): string => {
        if (rarityNum === 6) return "text-red-600 font-bold";
        if (rarityNum === 5) return "text-orange-500 font-semibold";
        if (rarityNum === 4) return "text-yellow-500 font-semibold";
        if (rarityNum === 3) return "text-purple-500";
        if (rarityNum === 2) return "text-green-600";
        if (rarityNum === 1) return "text-gray-500";
        return "text-gray-400";
    };

    // Function to get all relevant tags for an operator (based on old code logic)
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

    // Click handler for operator elements
    const handleOperatorClick = (op: Operator, outcomeIndex: number) => {
        const opId = op.id ?? op.name; // Use ID or fallback to name
        // Check if clicking the already selected operator
        if (selectedOpInfo?.outcomeIndex === outcomeIndex && selectedOpInfo?.operatorId === opId) {
            setSelectedOpInfo(null); // Hide tags
        } else {
            // Calculate and store the tags for the newly clicked operator
            const allTags = getOperatorDisplayTags(op);
            setSelectedOpInfo({
                outcomeIndex,
                operatorId: opId,
                allTags,
            });
        }
    };

    return (
        <div className="container mx-auto p-4">
            <h1 className="mt-2 text-2xl font-bold">Arknights Recruitment Calculator</h1>
            <p className="mb-4 text-xs md:text-sm">
                Calculate the probability of getting a specific operator or operators in a recruitment. Please note that the calculator may not have all operators available.
                <br />
                <b>Note:</b> This calculator uses calculations from{" "}
                <Link href={"https://github.com/akgcc/akgcc.github.io"} className="text-blue-500 hover:underline">
                    akgcc/akgcc.github.io
                </Link>
                . All credit for the recruitment calculations goes to them.
            </p>
            <Card>
                <CardContent>
                    <div className="mb-4 pt-2">
                        <h3 className="mb-2 text-lg font-semibold">Select Tags (Up to 5)</h3>
                        {isLoadingTags ? (
                            <div className="mb-4 flex h-72 w-full items-center justify-center rounded-md border p-4">
                                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                                <span className="ml-2">Loading tags...</span>
                            </div>
                        ) : errorTags ? (
                            <Alert variant="destructive" className="mb-4">
                                <AlertTitle>Error Loading Tags</AlertTitle>
                                <AlertDescription>{errorTags}</AlertDescription>
                            </Alert>
                        ) : (
                            <ScrollArea className="mb-4 h-72 w-full rounded-md border p-4">
                                {Object.entries(groupedTags).length > 0 ? (
                                    Object.entries(groupedTags).map(([type, tags]) => (
                                        <div key={type} className="mb-4">
                                            <h4 className="mb-2 font-medium text-primary">{type}</h4>
                                            <div className="grid grid-cols-2 gap-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5">
                                                {tags.map((tag) => (
                                                    <div key={tag.tagId} className="flex items-center space-x-2 rounded-md border p-2 transition-colors hover:bg-accent">
                                                        <Checkbox
                                                            id={tag.tagId}
                                                            checked={selectedTags.has(tag.tagId)}
                                                            onCheckedChange={(checked: boolean | "indeterminate") => handleTagChange(tag.tagId, checked)} // Added type here too
                                                            disabled={selectedTags.size >= 5 && !selectedTags.has(tag.tagId)}
                                                        />
                                                        <label htmlFor={tag.tagId} className={`text-sm font-medium leading-none ${selectedTags.size >= 5 && !selectedTags.has(tag.tagId) ? "cursor-not-allowed text-muted-foreground" : "cursor-pointer"}`}>
                                                            {tag.tagName}
                                                        </label>
                                                    </div>
                                                ))}
                                            </div>
                                        </div>
                                    ))
                                ) : (
                                    <p className="text-muted-foreground">No tags found or failed to load.</p>
                                )}
                            </ScrollArea>
                        )}
                        <Button onClick={resetSelection} variant="outline" disabled={selectedTags.size === 0}>
                            Reset Selection
                        </Button>
                    </div>

                    <Separator className="my-6" />

                    <div>
                        <h3 className="mb-2 text-lg font-semibold">Possible Operator Outcomes</h3>
                        <div className="relative min-h-[150px] rounded-md border bg-muted/50 p-4">
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
                                        <table className="w-full table-fixed border-collapse text-sm">
                                            <thead>
                                                <tr className="border-b">
                                                    <th className="w-1/3 px-2 py-2 text-left font-medium text-muted-foreground">Combination</th>
                                                    <th className="w-2/3 px-2 py-2 text-left font-medium text-muted-foreground">Operators</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {/* Map over the possible results */}
                                                {possibleOutcomes.map(
                                                    (outcome, outcomeIndex) =>
                                                        // Use outcome.operators instead of outcome.matches
                                                        outcome.operators &&
                                                        outcome.operators.length > 0 && (
                                                            <tr key={outcomeIndex} className="border-b align-top last:border-b-0 hover:bg-muted/20">
                                                                {/* Render logic for tags and operators remains the same */}
                                                                {/* Tags Column - Use outcome.label */}
                                                                <td className="p-2">
                                                                    <div className="flex flex-wrap gap-1">
                                                                        {/* Map over outcome.label (string[]) */}
                                                                        {outcome.label.map((tagName) => (
                                                                            <div key={tagName} className="whitespace-nowrap rounded bg-secondary px-1.5 py-0.5 text-xs font-medium text-secondary-foreground">
                                                                                {tagName}
                                                                            </div>
                                                                        ))}
                                                                    </div>
                                                                </td>
                                                                {/* Operators Column - Use outcome.operators */}
                                                                <td className="p-2">
                                                                    <div className="flex flex-wrap gap-x-2 gap-y-1">
                                                                        {/* Sort and map outcome.operators */}
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
                                                                                    <span key={opId} onClick={() => handleOperatorClick(op, outcomeIndex)} className={cn(getRarityColor(rarityNum), "cursor-pointer rounded px-1 py-0.5 hover:bg-primary/10", isSelected && "bg-primary/20 ring-1 ring-primary")} title="Click to see all tags">
                                                                                        {op.name} ({rarityNum}★)
                                                                                        {/* Use op.recruitOnly directly */}
                                                                                        {op.recruitOnly && <span className="ml-0.5 text-xs text-blue-500 opacity-80">*</span>}
                                                                                    </span>
                                                                                );
                                                                            })}
                                                                    </div>
                                                                    {/* Conditionally display tags for the selected operator */}
                                                                    {selectedOpInfo?.outcomeIndex === outcomeIndex && (
                                                                        <div className="mt-2 flex flex-wrap gap-1 border-t pt-2">
                                                                            {selectedOpInfo.allTags.map((tagName) => (
                                                                                <div key={tagName} className="whitespace-nowrap rounded bg-accent px-1.5 py-0.5 text-xs text-accent-foreground">
                                                                                    {tagName}
                                                                                </div>
                                                                            ))}
                                                                        </div>
                                                                    )}
                                                                </td>
                                                            </tr>
                                                        ),
                                                )}
                                            </tbody>
                                        </table>
                                    ) : (
                                        <p className="text-muted-foreground">No guaranteed outcomes found for the selected tags.</p>
                                    )}
                                </>
                            )}
                        </div>
                    </div>
                </CardContent>
            </Card>
        </div>
    );
};

export default RecruitmentCalculator;
