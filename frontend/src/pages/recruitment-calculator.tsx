"use client";

import React, { useState, useEffect, useCallback, useRef } from "react";
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
import { List, User, Search } from "lucide-react"; // Import icons for buttons
import Image from "next/image"; // Import next/image
import { HoverCard, HoverCardContent, HoverCardTrigger } from "~/components/ui/hover-card"; // Import HoverCard components
import { Badge } from "~/components/ui/badge"; // Import Badge component
import { Input } from "~/components/ui/input"; // Import Input component
import Head from "next/head";
import type { ApiResponse, ErrorResponse, OperatorOutcome } from "~/types/impl/frontend/impl/recruitment-calculator";
import type { SelectedOpInfo } from "~/types/impl/frontend/impl/recruitment-calculator";
import type { Tag } from "~/types/impl/frontend/impl/recruitment-calculator";
import type { GroupedTags } from "~/types/impl/frontend/impl/recruitment-calculator";

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
    // State for toggling view mode
    const [viewMode, setViewMode] = useState<"text" | "profile">("text");
    // State for tag search query
    const [tagSearchQuery, setTagSearchQuery] = useState<string>("");

    // State for resizable tag area
    const initialTagAreaHeight = 288; // Corresponds to h-72
    const [tagAreaHeight, setTagAreaHeight] = useState<number>(initialTagAreaHeight);
    const [isDragging, setIsDragging] = useState<boolean>(false);
    const dragStartRef = useRef<{ y: number; height: number } | null>(null);
    const scrollAreaRef = useRef<HTMLDivElement>(null); // Ref for the ScrollArea container

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

    // Drag handling effects
    useEffect(() => {
        const handleMouseMove = (e: MouseEvent) => {
            if (!isDragging || !dragStartRef.current) return;

            const deltaY = e.clientY - dragStartRef.current.y;
            let newHeight = dragStartRef.current.height + deltaY;

            // Clamp height (min: initial, max: 75% of viewport height)
            const maxHeight = window.innerHeight * 0.75;
            newHeight = Math.max(initialTagAreaHeight, Math.min(newHeight, maxHeight));

            setTagAreaHeight(newHeight);
        };

        const handleMouseUp = () => {
            if (isDragging) {
                setIsDragging(false);
                dragStartRef.current = null;
                // Optional: Remove text selection cursor style globally if applied
                document.body.style.cursor = "";
                document.body.style.userSelect = "";
            }
        };

        if (isDragging) {
            // Optional: Change cursor globally to indicate dragging
            document.body.style.cursor = "ns-resize";
            document.body.style.userSelect = "none"; // Prevent text selection during drag
            window.addEventListener("mousemove", handleMouseMove);
            window.addEventListener("mouseup", handleMouseUp);
        } else {
            window.removeEventListener("mousemove", handleMouseMove);
            window.removeEventListener("mouseup", handleMouseUp);
        }

        // Cleanup listeners on component unmount or when isDragging changes
        return () => {
            window.removeEventListener("mousemove", handleMouseMove);
            window.removeEventListener("mouseup", handleMouseUp);
            // Reset global styles if component unmounts while dragging
            if (document.body.style.cursor === "ns-resize") {
                document.body.style.cursor = "";
            }
            if (document.body.style.userSelect === "none") {
                document.body.style.userSelect = "";
            }
        };
    }, [isDragging]); // Re-run effect when isDragging changes

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

    const handleMouseDownOnGrabber = (e: React.MouseEvent<HTMLDivElement>) => {
        setIsDragging(true);
        dragStartRef.current = {
            y: e.clientY,
            height: tagAreaHeight,
        };
        // Prevent default text selection behavior during drag
        e.preventDefault();
    };

    const resetSelection = () => {
        setSelectedTags(new Set());
        setSelectedOpInfo(null); // Clear selected op tags on reset
    };

    // Helper function to get rarity color - adjust as needed
    const getRarityColor = (rarityNum: number): string => {
        if (rarityNum === 6) return "text-orange-500 font-bold";
        if (rarityNum === 5) return "text-yellow-500 font-semibold";
        if (rarityNum === 4) return "text-purple-500 font-semibold";
        if (rarityNum === 3) return "text-blue-500";
        if (rarityNum === 2) return "text-green-600";
        if (rarityNum === 1) return "text-gray-500";
        return "text-gray-400";
    };

    // Helper function to get rarity border color
    const getRarityBorderColor = (rarityNum: number): string => {
        if (rarityNum === 6) return "border border-orange-400/60 shadow-[0_0_6px_theme(colors.orange.400/50)]"; // Top Op
        if (rarityNum === 5) return "border border-yellow-400/60 shadow-[0_0_6px_theme(colors.yellow.400/50)]"; // Senior Op
        if (rarityNum === 4) return "border border-purple-400/60 shadow-[0_0_6px_theme(colors.purple.400/50)]"; // 4*
        if (rarityNum === 3) return "border border-blue-400/60 shadow-[0_0_6px_theme(colors.blue.400/50)]"; // 3*
        if (rarityNum === 2) return "border border-green-500/60 shadow-[0_0_6px_theme(colors.green.500/50)]"; // 2*
        if (rarityNum === 1) return "border border-gray-400/60 shadow-[0_0_6px_theme(colors.gray.400/50)]"; // 1*
        return "border border-gray-300/60 shadow-[0_0_6px_theme(colors.gray.300/50)]"; // Default/Robot
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

    // Click handler for operator elements (now only used in text view)
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
            <Head>
                <title>Arknights Recruitment Calculator</title>
                <meta name="title" content="Arknights Recruitment Calculator" />
                <meta name="description" content="Calculate the probability of getting a specific operator or operators in a recruitment." />
                <link rel="icon" href="/favicon.ico" />
            </Head>
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
                                <>
                                    <div className="relative mb-3 flex items-center gap-2">
                                        {" "}
                                        {/* Flex container for input and icons */}
                                        <Search className="absolute left-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" /> {/* Search Icon */}
                                        <Input
                                            type="text"
                                            placeholder="Search tags..."
                                            value={tagSearchQuery}
                                            onChange={(e) => setTagSearchQuery(e.target.value)}
                                            className="flex-grow pl-8" // Add padding for the icon and allow input to grow
                                        />
                                    </div>
                                    <ScrollArea ref={scrollAreaRef} className="w-full overflow-auto rounded-t-md border border-b-0 p-4" style={{ height: `${tagAreaHeight}px` }}>
                                        {Object.entries(groupedTags).length > 0 ? (
                                            Object.entries(groupedTags).map(([type, tags]) => {
                                                const filteredTags = tags.filter((tag) => tag.tagName.toLowerCase().includes(tagSearchQuery.toLowerCase()));

                                                if (filteredTags.length === 0) {
                                                    return null; // Don't render the group if no tags match the search
                                                }

                                                return (
                                                    <div key={type} className="mb-4">
                                                        <h4 className="mb-2 font-medium text-primary">{type}</h4>
                                                        <div className="grid grid-cols-2 gap-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5">
                                                            {filteredTags.map((tag) => {
                                                                const isDisabled = selectedTags.size >= 5 && !selectedTags.has(tag.tagId);
                                                                const isChecked = selectedTags.has(tag.tagId);

                                                                return (
                                                                    <div
                                                                        key={tag.tagId}
                                                                        className={cn(
                                                                            "flex items-center space-x-2 rounded-md border p-2 transition-colors hover:bg-accent",
                                                                            !isDisabled && "cursor-pointer", // Add cursor-pointer if not disabled
                                                                            isDisabled && "cursor-not-allowed opacity-70", // Style disabled state
                                                                        )}
                                                                        onClick={() => {
                                                                            if (!isDisabled) {
                                                                                handleTagChange(tag.tagId, !isChecked); // Toggle based on current state
                                                                            }
                                                                        }}
                                                                    >
                                                                        <Checkbox
                                                                            id={tag.tagId}
                                                                            checked={isChecked}
                                                                            // Keep onCheckedChange for accessibility and direct checkbox clicks
                                                                            onCheckedChange={(checked: boolean | "indeterminate") => handleTagChange(tag.tagId, checked)}
                                                                            disabled={isDisabled}
                                                                            aria-labelledby={`${tag.tagId}-label`} // Associate checkbox with label for screen readers
                                                                        />
                                                                        <label
                                                                            id={`${tag.tagId}-label`} // Add id for aria-labelledby
                                                                            htmlFor={tag.tagId} // Keep htmlFor for label-checkbox association
                                                                            className={`select-none text-sm font-medium leading-none ${isDisabled ? "cursor-not-allowed text-muted-foreground" : "cursor-pointer"}`} // Remove explicit cursor-pointer here
                                                                            // Prevent label click from propagating to the div's onClick
                                                                            onClick={(e) => e.stopPropagation()}
                                                                        >
                                                                            {tag.tagName}
                                                                        </label>
                                                                    </div>
                                                                );
                                                            })}
                                                        </div>
                                                    </div>
                                                );
                                            })
                                        ) : (
                                            <p className="text-muted-foreground">No tags found or failed to load.</p>
                                        )}
                                    </ScrollArea>
                                    {/* Draggable Grabber Handle */}
                                    <div onMouseDown={handleMouseDownOnGrabber} className="mb-4 h-2 w-full cursor-ns-resize rounded-b-md border border-t-0 bg-muted transition-colors hover:bg-muted-foreground/20 active:bg-muted-foreground/30" title="Drag to resize tags area">
                                        {/* Optional: Add visual indicator like dots */}
                                        <div className="flex h-full items-center justify-center">
                                            <span className="h-1 w-8 rounded-full bg-muted-foreground/40"></span>
                                        </div>
                                    </div>
                                </>
                            )}
                            <Button onClick={resetSelection} variant="outline" disabled={selectedTags.size === 0}>
                                Reset Selection
                            </Button>
                        </div>

                        <Separator className="my-6" />

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
                                                                                        <span key={opId} onClick={() => handleOperatorClick(op, outcomeIndex)} className={cn(getRarityColor(rarityNum), "cursor-pointer rounded px-1 py-0.5 hover:bg-primary/10", isSelected && "bg-primary/15 ring-1 ring-primary/25 transition-all duration-150")} title="Click to see all tags">
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
                    </CardContent>
                </Card>
            </div>
        </>
    );
};

export default RecruitmentCalculator;
