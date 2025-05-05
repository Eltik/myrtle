import React, { useState, useEffect, useCallback } from "react";
import { Card, CardContent } from "~/components/ui/card";
import { Separator } from "~/components/ui/separator";
import Link from "next/link";
import type { ApiResponse, ErrorResponse, OperatorOutcome } from "~/types/impl/frontend/impl/recruitment-calculator";
import type { SelectedOpInfo } from "~/types/impl/frontend/impl/recruitment-calculator";
import type { Tag } from "~/types/impl/frontend/impl/recruitment-calculator";
import type { GroupedTags } from "~/types/impl/frontend/impl/recruitment-calculator";
import { TagSelector } from "~/components/recruitment-calculator/impl/tag-selector";
import { RecruitOutcome } from "~/components/recruitment-calculator/impl/recruit-outcome";

export const Recruitment = () => {
    const [allTagsMap, setAllTagsMap] = useState<Record<string, Tag>>({}); // For easy lookup
    const [groupedTags, setGroupedTags] = useState<GroupedTags>({});
    const [selectedTags, setSelectedTags] = useState<Set<string>>(new Set());
    // State is now an array again
    const [possibleOutcomes, setPossibleOutcomes] = useState<OperatorOutcome[]>([]);
    const [isLoadingTags, setIsLoadingTags] = useState<boolean>(true);
    const [isLoadingOutcomes, setIsLoadingOutcomes] = useState<boolean>(false);
    const [errorTags, setErrorTags] = useState<string | null>(null);
    const [errorOutcomes, setErrorOutcomes] = useState<string | null>(null);
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

    return (
        <>
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
                        <TagSelector isLoadingTags={isLoadingTags} errorTags={errorTags} groupedTags={groupedTags} selectedTags={selectedTags} setSelectedTags={setSelectedTags} setSelectedOpInfo={setSelectedOpInfo} />
                        <Separator className="my-6" />
                        <RecruitOutcome allTagsMap={allTagsMap} selectedOpInfo={selectedOpInfo} setSelectedOpInfo={setSelectedOpInfo} possibleOutcomes={possibleOutcomes} isLoadingOutcomes={isLoadingOutcomes} errorOutcomes={errorOutcomes} selectedTags={selectedTags} />
                    </CardContent>
                </Card>
            </div>
        </>
    );
};
