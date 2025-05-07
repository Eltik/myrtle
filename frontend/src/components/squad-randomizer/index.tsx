import { useState, useEffect, useCallback, useRef } from "react";
import { Input } from "~/components/ui/input";
import { Button } from "~/components/ui/button";
import { ScrollArea } from "~/components/ui/scroll-area";
import type { Operator } from "~/types/impl/api/static/operator";
import { OperatorProfession } from "~/types/impl/api/static/operator";
import { ListIcon, GridIcon, ArrowUpIcon, ChevronUpIcon } from "lucide-react";
import { rarityToNumber } from "~/helper"; // Import helpers
import { renderOperatorListItem } from "~/components/squad-randomizer/impl/render-operator-list";
import { renderOperatorGridItem } from "~/components/squad-randomizer/impl/render-operator-grid-item";
import { TopSection } from "~/components/squad-randomizer/impl/top-section";
import { RarityFilter } from "~/components/squad-randomizer/impl/rarity-filter";
import { ProfessionFilter } from "~/components/squad-randomizer/impl/profession-filter";
import { TagFilter } from "~/components/squad-randomizer/impl/tag-filter";
import { DisplayFilters } from "~/components/squad-randomizer/impl/display-filters";
import { motion, AnimatePresence } from "framer-motion";

const Randomizer = () => {
    const [allOperators, setAllOperators] = useState<Operator[]>([]);
    const [filteredOperators, setFilteredOperators] = useState<Operator[]>([]);
    const [randomizedSquad, setRandomizedSquad] = useState<Operator[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [squadSize, setSquadSize] = useState(12);
    const [searchTerm, setSearchTerm] = useState("");
    const [excludedOperators, setExcludedOperators] = useState<Set<string>>(new Set());
    const [filterRarityNumeric, setFilterRarityNumeric] = useState<Set<number>>(new Set([1, 2, 3, 4, 5, 6]));
    const [filterProfession, setFilterProfession] = useState<Set<OperatorProfession>>(new Set(Object.values(OperatorProfession)));
    const [viewMode, setViewMode] = useState<"list" | "grid">("grid");
    const [allTags, setAllTags] = useState<Set<string>>(new Set());
    const [selectedTags, setSelectedTags] = useState<Set<string>>(new Set());
    const [showScrollTop, setShowScrollTop] = useState(false);

    const [visibleOperatorCount, setVisibleOperatorCount] = useState(40);

    const observer = useRef<IntersectionObserver | null>(null);
    const lastCharacterRef = useCallback(
        (node: HTMLDivElement) => {
            if (observer.current) observer.current.disconnect();
            observer.current = new IntersectionObserver((entries) => {
                if (entries[0]?.isIntersecting && visibleOperatorCount < filteredOperators.length) {
                    setVisibleOperatorCount((prevCount) => Math.min(prevCount + 40, filteredOperators.length));
                }
            });
            if (node) observer.current.observe(node);
        },
        [visibleOperatorCount, filteredOperators.length],
    );

    const [isFiltersCollapsed, setIsFiltersCollapsed] = useState(false);

    const filterVariants = {
        expanded: {
            height: "auto",
            opacity: 1,
            transition: {
                height: {
                    duration: 0.3,
                    ease: [0.4, 0, 0.2, 1],
                },
                opacity: {
                    duration: 0.2,
                    delay: 0.1,
                },
            },
        },
        collapsed: {
            height: 0,
            opacity: 0,
            transition: {
                height: {
                    duration: 0.3,
                    ease: [0.4, 0, 0.2, 1],
                },
                opacity: {
                    duration: 0.2,
                },
            },
        },
    };

    const buttonVariants = {
        expanded: {
            rotate: 0,
            transition: { duration: 0.3, ease: [0.4, 0, 0.2, 1] },
        },
        collapsed: {
            rotate: 180,
            transition: { duration: 0.3, ease: [0.4, 0, 0.2, 1] },
        },
    };

    useEffect(() => {
        const fetchOperators = async () => {
            setIsLoading(true);
            setError(null);
            try {
                const response = await fetch("/api/static", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({
                        type: "operators",
                        fields: ["id", "name", "rarity", "profession", "subProfessionId", "tagList"],
                    }),
                });

                if (!response.ok) {
                    throw new Error(`HTTP error ${response.status}`);
                }

                const data = (await response.json()) as { data: Operator[] };

                if (!data.data) {
                    throw new Error("No operator data received");
                }

                // Filter out operators without an ID and sort
                const validOperators = data.data.filter((op) => op.id);

                // Extract all unique tags
                const uniqueTags = new Set<string>();
                validOperators.forEach((op) => {
                    op.tagList?.forEach((tag) => uniqueTags.add(tag));
                });
                setAllTags(uniqueTags);

                const sortedData = validOperators.sort((a, b) => {
                    const rarityA = rarityToNumber(a.rarity);
                    const rarityB = rarityToNumber(b.rarity);
                    if (rarityB !== rarityA) {
                        return rarityB - rarityA; // Sort descending by numeric rarity
                    }
                    return a.name.localeCompare(b.name);
                });

                setAllOperators(sortedData);
                setFilteredOperators(sortedData);
            } catch (err) {
                console.error("Failed to fetch operators:", err);
                setError(err instanceof Error ? err.message : "An unknown error occurred");
            } finally {
                setIsLoading(false);
            }
        };

        void fetchOperators();
    }, []);

    // Update filteredOperators whenever filters change
    useEffect(() => {
        let currentOperators = [...allOperators];

        // Filter by search term (name or ID)
        if (searchTerm) {
            currentOperators = currentOperators.filter((op) => op.name.toLowerCase().includes(searchTerm.toLowerCase()) || op.id?.toLowerCase().includes(searchTerm.toLowerCase()));
        }

        // Apply numeric rarity filter
        currentOperators = currentOperators.filter((op) => filterRarityNumeric.has(rarityToNumber(op.rarity)));

        // Apply profession filter
        currentOperators = currentOperators.filter((op) => filterProfession.has(op.profession));

        // Apply tag filter (must have ALL selected tags)
        if (selectedTags.size > 0) {
            currentOperators = currentOperators.filter((op) => {
                const opTags = new Set(op.tagList ?? []);
                return Array.from(selectedTags).every((filterTag) => opTags.has(filterTag));
            });
        }

        setFilteredOperators(currentOperators);
    }, [allOperators, searchTerm, filterRarityNumeric, filterProfession, selectedTags]);

    const handleSquadSizeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const size = parseInt(e.target.value, 10);
        if (!isNaN(size) && size > 0) {
            setSquadSize(size);
        }
    };

    const handleRandomize = useCallback(() => {
        // Filter out excluded operators directly
        const candidates = filteredOperators.filter((op) => op.id && !excludedOperators.has(op.id));

        // Shuffle candidates
        const shuffledCandidates = [...candidates].sort(() => 0.5 - Math.random());

        // Take the required number based on squadSize
        const finalSquad = shuffledCandidates.slice(0, squadSize);

        // Sort final squad by rarity
        finalSquad.sort((a, b) => rarityToNumber(b.rarity) - rarityToNumber(a.rarity));

        setRandomizedSquad(finalSquad);
    }, [filteredOperators, excludedOperators, squadSize]);

    // Add scroll handler
    useEffect(() => {
        const handleScroll = () => {
            const scrollY = window.scrollY;
            setShowScrollTop(scrollY > 300); // Show button when scrolled more than 300px
        };

        window.addEventListener("scroll", handleScroll);
        return () => window.removeEventListener("scroll", handleScroll);
    }, []);

    const scrollToTop = () => {
        window.scrollTo({
            top: 0,
            behavior: "smooth",
        });
    };

    return (
        <div className="container mx-auto flex flex-col p-4">
            <h1 className="mb-6 flex-shrink-0 text-3xl font-bold">Arknights Squad Randomizer</h1>

            {error && <p className="mb-4 flex-shrink-0 text-red-500">Error loading operators: {error}</p>}

            <TopSection allOperators={allOperators} squadSize={squadSize} handleSquadSizeChange={handleSquadSizeChange} handleRandomize={handleRandomize} isLoading={isLoading} filteredOperators={filteredOperators} excludedOperators={excludedOperators} randomizedSquad={randomizedSquad} setExcludedOperators={setExcludedOperators} />

            {/* Bottom Section: Filters and Operator List */}
            <div className="flex min-h-0 flex-grow flex-col border-t pt-4">
                <h2 className="mb-3 flex-shrink-0 text-xl font-semibold">Filter & Select Operators</h2>
                {/* Filter Controls Row */}
                <div className="sticky top-14 z-30 rounded-b-md bg-background/50 shadow-md backdrop-blur-md">
                    <AnimatePresence initial={false}>
                        <motion.div initial="collapsed" animate={isFiltersCollapsed ? "collapsed" : "expanded"} variants={filterVariants} className="overflow-hidden">
                            <div className="mb-1 flex flex-shrink-0 flex-wrap items-center gap-2 p-2">
                                <Input type="text" placeholder="Search operators..." value={searchTerm} onChange={(e) => setSearchTerm(e.target.value)} className="h-9 max-w-xs bg-background" />
                                <RarityFilter filterRarityNumeric={filterRarityNumeric} setFilterRarityNumeric={setFilterRarityNumeric} />
                                <ProfessionFilter filterProfession={filterProfession} setFilterProfession={setFilterProfession} />
                                <TagFilter selectedTags={selectedTags} setSelectedTags={setSelectedTags} allTags={allTags} />

                                {/* View Toggle Buttons */}
                                <div className="ml-auto flex items-center space-x-1">
                                    <Button variant={viewMode === "list" ? "secondary" : "outline"} size="icon" onClick={() => setViewMode("list")} title="List View">
                                        <ListIcon className="h-4 w-4" />
                                    </Button>
                                    <Button variant={viewMode === "grid" ? "secondary" : "outline"} size="icon" onClick={() => setViewMode("grid")} title="Grid View">
                                        <GridIcon className="h-4 w-4" />
                                    </Button>
                                </div>
                                <p className="mt-2 flex-shrink-0 text-sm text-muted-foreground">
                                    Showing {filteredOperators.filter((op) => op.id?.startsWith("char_")).length} / {allOperators.filter((op) => op.id?.startsWith("char_")).length} operators.
                                    {excludedOperators.size > 0 && ` ${excludedOperators.size} excluded.`}
                                </p>
                            </div>
                            <DisplayFilters filterRarityNumeric={filterRarityNumeric} filterProfession={filterProfession} selectedTags={selectedTags} filteredOperators={filteredOperators} excludedOperators={excludedOperators} setFilterRarityNumeric={setFilterRarityNumeric} setFilterProfession={setFilterProfession} setSelectedTags={setSelectedTags} setExcludedOperators={setExcludedOperators} />
                        </motion.div>
                    </AnimatePresence>
                    <Button onClick={() => setIsFiltersCollapsed(!isFiltersCollapsed)} className="flex w-full items-center justify-center gap-2 rounded-b-md bg-muted/40 text-sm text-muted-foreground transition-all hover:bg-muted/70 hover:text-foreground">
                        <motion.div variants={buttonVariants} animate={isFiltersCollapsed ? "collapsed" : "expanded"}>
                            <ChevronUpIcon className="h-4 w-4" />
                        </motion.div>
                    </Button>
                </div>

                {/* Operator List/Grid Area */}
                <ScrollArea className="h-full min-h-[300px] flex-grow rounded-md border">
                    <div className="p-2">{isLoading ? <p className="text-center text-muted-foreground">Loading operators...</p> : filteredOperators.length > 0 ? viewMode === "list" ? <div className="space-y-0.5">{filteredOperators.slice(0, visibleOperatorCount).map((op, index) => renderOperatorListItem(op, excludedOperators, setExcludedOperators, index === visibleOperatorCount - 1 ? lastCharacterRef : null))}</div> : <div className="grid grid-cols-3 gap-2 sm:grid-cols-4 md:grid-cols-6 lg:grid-cols-7 xl:grid-cols-8">{filteredOperators.slice(0, visibleOperatorCount).map((op, index) => renderOperatorGridItem(op, excludedOperators, setExcludedOperators, index === visibleOperatorCount - 1 ? lastCharacterRef : null))}</div> : <p className="text-center text-muted-foreground">No operators match filters.</p>}</div>
                </ScrollArea>
            </div>

            {/* Add Scroll to Top Button */}
            <Button onClick={scrollToTop} className={`fixed bottom-8 right-8 z-50 rounded-full shadow-lg transition-all duration-300 ${showScrollTop ? "translate-y-0 opacity-100" : "pointer-events-none translate-y-4 opacity-0"}`} size="icon" variant="secondary">
                <ArrowUpIcon className="h-4 w-4" />
            </Button>
        </div>
    );
};

export default Randomizer;
