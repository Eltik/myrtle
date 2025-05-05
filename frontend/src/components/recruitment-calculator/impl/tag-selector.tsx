import { Loader2, Search } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { Alert, AlertDescription, AlertTitle } from "~/components/ui/alert";
import { Button } from "~/components/ui/button";
import { Checkbox } from "~/components/ui/checkbox";
import { Input } from "~/components/ui/input";
import { ScrollArea } from "~/components/ui/scroll-area";
import { cn } from "~/lib/utils";
import type { GroupedTags, SelectedOpInfo } from "~/types/impl/frontend/impl/recruitment-calculator";

export const TagSelector = ({ isLoadingTags, errorTags, groupedTags, selectedTags, setSelectedTags, setSelectedOpInfo }: { isLoadingTags: boolean; errorTags: string | null; groupedTags: GroupedTags; selectedTags: Set<string>; setSelectedTags: React.Dispatch<React.SetStateAction<Set<string>>>; setSelectedOpInfo: (selectedOpInfo: SelectedOpInfo | null) => void }) => {
    const [tagSearchQuery, setTagSearchQuery] = useState<string>("");

    const initialTagAreaHeight = 288; // Corresponds to h-72
    const [tagAreaHeight, setTagAreaHeight] = useState<number>(initialTagAreaHeight);
    const [isDragging, setIsDragging] = useState<boolean>(false);
    const dragStartRef = useRef<{ y: number; height: number } | null>(null);
    const scrollAreaRef = useRef<HTMLDivElement>(null); // Ref for the ScrollArea container

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

    return (
        <>
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
        </>
    );
};
