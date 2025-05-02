import { useState, useEffect, useCallback } from "react";
import { Input } from "~/components/ui/input";
import { Button } from "~/components/ui/button";
import { Label } from "~/components/ui/label";
import { ScrollArea } from "~/components/ui/scroll-area";
import type { Operator } from "~/types/impl/api/static/operator";
import { OperatorRarity, OperatorProfession } from "~/types/impl/api/static/operator";
import { ListIcon, GridIcon, CheckIcon, PlusCircleIcon, StarIcon, XCircleIcon } from "lucide-react";
import { Command, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList, CommandSeparator } from "~/components/ui/command";
import { Popover, PopoverContent, PopoverTrigger } from "~/components/ui/popover";
import { Badge } from "~/components/ui/badge";
import { cn } from "~/lib/utils";
import Image from "next/image";
import { rarityToNumber, formatProfession, formatSubProfession } from "~/helper"; // Import helpers

// Operator type remains the same from import
type OperatorOption = Operator;

// Use OperatorProfession enum keys
const PROFESSION_MAP: Record<OperatorProfession, string> = {
    [OperatorProfession.VANGUARD]: "Vanguard",
    [OperatorProfession.GUARD]: "Guard",
    [OperatorProfession.SNIPER]: "Sniper",
    [OperatorProfession.DEFENDER]: "Defender",
    [OperatorProfession.MEDIC]: "Medic",
    [OperatorProfession.SUPPORTER]: "Supporter",
    [OperatorProfession.CASTER]: "Caster",
    [OperatorProfession.SPECIALIST]: "Specialist",
};

// Map OperatorRarity enum to display colors
const RARITY_COLORS: Record<OperatorRarity, string> = {
    [OperatorRarity.oneStar]: "text-gray-500",
    [OperatorRarity.twoStar]: "text-green-600",
    [OperatorRarity.threeStar]: "text-blue-500",
    [OperatorRarity.fourStar]: "text-purple-500",
    [OperatorRarity.fiveStar]: "text-yellow-500",
    [OperatorRarity.sixStar]: "text-orange-500",
};

// Helper to get star icons
const renderStars = (rarityEnum: OperatorRarity) => {
    const rarityNum = rarityToNumber(rarityEnum);
    return (
        <div className="flex items-center">
            {Array.from({ length: rarityNum }, (_, i) => (
                <StarIcon key={i} className={`h-3 w-3 ${RARITY_COLORS[rarityEnum]?.replace("text-", "fill-") ?? "fill-gray-400"}`} />
            ))}
        </div>
    );
};

const SquadRandomizer = () => {
    const [allOperators, setAllOperators] = useState<OperatorOption[]>([]);
    const [filteredOperators, setFilteredOperators] = useState<OperatorOption[]>([]);
    const [randomizedSquad, setRandomizedSquad] = useState<OperatorOption[]>([]);
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
    const [tagPopoverOpen, setTagPopoverOpen] = useState(false);
    const [rarityPopoverOpen, setRarityPopoverOpen] = useState(false);
    const [professionPopoverOpen, setProfessionPopoverOpen] = useState(false);

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

    const handleToggleExclude = (operatorId: string | undefined) => {
        if (!operatorId) return; // Ignore if id is undefined
        setExcludedOperators((prev) => {
            const newSet = new Set(prev);
            if (newSet.has(operatorId)) {
                newSet.delete(operatorId);
            } else {
                newSet.add(operatorId);
            }
            return newSet;
        });
    };

    const handleSquadSizeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const size = parseInt(e.target.value, 10);
        if (!isNaN(size) && size > 0) {
            setSquadSize(size);
        }
    };

    // Use numeric rarity (0-5) for handler
    const handleRarityChange = (numericRarity: number) => {
        setFilterRarityNumeric((prev) => {
            const newSet = new Set(prev);
            if (newSet.has(numericRarity)) {
                newSet.delete(numericRarity);
            } else {
                newSet.add(numericRarity);
            }
            return newSet.size === 0 ? new Set([1, 2, 3, 4, 5, 6]) : newSet;
        });
    };

    const handleProfessionChange = (profession: OperatorProfession) => {
        setFilterProfession((prev) => {
            const newSet = new Set(prev);
            if (newSet.has(profession)) {
                newSet.delete(profession);
            } else {
                newSet.add(profession);
            }
            return newSet.size === 0 ? new Set(Object.values(OperatorProfession)) : newSet;
        });
    };

    // Handler for tag selection/deselection
    const handleTagSelect = (tag: string) => {
        setSelectedTags((prev) => {
            const newSet = new Set(prev);
            if (newSet.has(tag)) {
                newSet.delete(tag);
            } else {
                newSet.add(tag);
            }
            return newSet;
        });
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

    // Updated List View Renderer
    const renderOperatorListItem = (op: OperatorOption) => {
        if (!op.id) return null;
        const isExcluded = excludedOperators.has(op.id);
        const displayRarityColor = RARITY_COLORS[op.rarity] ?? "text-white";
        const displayProfession = formatProfession(op.profession);
        const displaySubProfession = formatSubProfession(op.subProfessionId);
        const imageUrl = `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/avatar/${op.id}.png`;

        return (
            <div key={op.id} className={cn("flex items-center justify-between border-b p-2 last:border-b-0", isExcluded && "bg-muted/30 opacity-60")}>
                <div className="mr-2 flex min-w-0 flex-1 items-center space-x-3">
                    <Image src={imageUrl} alt={op.name} width={40} height={40} className={cn("flex-shrink-0 rounded-full", isExcluded && "grayscale")} unoptimized />
                    <div className="min-w-0 flex-1 overflow-hidden">
                        <p className={`truncate font-semibold ${displayRarityColor}`}>{op.name}</p>
                        <p className="truncate text-xs text-muted-foreground">
                            {renderStars(op.rarity)} {displaySubProfession} <span className="mx-1">|</span> {displayProfession}
                        </p>
                        <div className="mt-1 flex flex-wrap gap-1">
                            {op.tagList?.slice(0, 4).map((tag) => (
                                <Badge key={tag} variant="secondary" className="px-1 py-0 text-xs">
                                    {tag}
                                </Badge>
                            ))}
                        </div>
                    </div>
                </div>
                <div className="flex flex-shrink-0 items-center space-x-1 sm:space-x-2">
                    <Button
                        variant={isExcluded ? "secondary" : "outline"}
                        size="sm"
                        onClick={() => {
                            console.log(`Button clicked for ${op.name} (ID: ${op.id})`);
                            handleToggleExclude(op.id);
                        }}
                        title={isExcluded ? "Allow in squad" : "Exclude from squad"}
                        className={cn("h-auto px-2 py-1 text-xs", isExcluded ? "text-muted-foreground hover:bg-muted hover:text-foreground" : "text-destructive hover:bg-destructive/10")}
                    >
                        {isExcluded ? "Allow" : "Exclude"}
                    </Button>
                </div>
            </div>
        );
    };

    // Render helper for operator grid items (Grid View) - Enhanced
    const renderOperatorGridItem = (op: OperatorOption) => {
        if (!op.id) return null;
        const isExcluded = excludedOperators.has(op.id);
        const displayRarityColor = RARITY_COLORS[op.rarity] ?? "text-white";
        const displayProfession = formatProfession(op.profession);
        const imageUrl = `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/portrait/${op.id}_1.png`;
        const rarityNum = rarityToNumber(op.rarity);

        return (
            <div key={op.id} className={cn("group relative aspect-[2/3] cursor-pointer overflow-hidden rounded-md border border-muted/50 bg-card shadow-sm transition-all duration-150", isExcluded ? "opacity-50 grayscale" : "hover:border-foreground/50 hover:shadow-md")} onClick={() => handleToggleExclude(op.id)} title={isExcluded ? `Click to Allow ${op.name}` : `Click to Exclude ${op.name}`}>
                <Image src={imageUrl} alt={op.name} fill sizes="(max-width: 640px) 30vw, (max-width: 1024px) 15vw, 10vw" className={cn("object-cover transition-transform duration-150", !isExcluded && "group-hover:scale-105")} loading="lazy" unoptimized />
                <div className="absolute inset-x-0 bottom-0 z-10 bg-gradient-to-t from-black/80 via-black/50 to-transparent p-2 pt-4">
                    <p className={`truncate text-sm font-semibold ${displayRarityColor}`}>{op.name}</p>
                    <p className="truncate text-xs text-gray-300">{displayProfession}</p>
                    <div className="text-xs text-yellow-300">{Array(rarityNum).fill("★").join("")}</div>
                </div>
                {isExcluded && (
                    <div className="absolute inset-0 z-20 flex items-center justify-center bg-black/40">
                        <XCircleIcon className="h-8 w-8 text-destructive opacity-80" />
                    </div>
                )}
            </div>
        );
    };

    return (
        <div className="container mx-auto flex h-screen flex-col p-4">
            <h1 className="mb-6 flex-shrink-0 text-3xl font-bold">Arknights Squad Randomizer</h1>

            {error && <p className="mb-4 flex-shrink-0 text-red-500">Error loading operators: {error}</p>}

            {/* Top Section: Options and Results */}
            <div className="mb-6 grid flex-shrink-0 grid-cols-1 gap-6 md:grid-cols-3">
                {/* Column 1: Options */}
                <div className="space-y-4 md:col-span-1">
                    <h2 className="mb-3 text-xl font-semibold">Options</h2>
                    <div>
                        <Label htmlFor="squadSize">Squad Size</Label>
                        <Input id="squadSize" type="number" value={squadSize} onChange={handleSquadSizeChange} min="1" max={allOperators.length > 0 ? allOperators.length : 12} className="mt-1" />
                    </div>
                    <Button onClick={handleRandomize} disabled={isLoading || filteredOperators.filter((op) => op.id && !excludedOperators.has(op.id)).length < squadSize} className="w-full">
                        Randomize Squad
                    </Button>
                </div>

                {/* Column 2 & 3: Randomized Squad */}
                <div className="md:col-span-2">
                    <h2 className="mb-3 text-xl font-semibold">
                        Randomized Squad ({randomizedSquad.length} / {squadSize})
                    </h2>
                    {randomizedSquad.length > 0 ? (
                        <div className="grid grid-cols-3 gap-2 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-7">
                            {randomizedSquad.map((op) => {
                                if (!op?.id) return null;
                                const displayRarityColor = RARITY_COLORS[op.rarity] ?? "text-white";
                                const displayProfession = formatProfession(op.profession);
                                const imageUrl = `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/avatar/${op.id}.png`;
                                const borderClass = RARITY_COLORS[op.rarity] ?? "border-gray-500";

                                return (
                                    <div key={op.id} className={cn("relative aspect-square overflow-hidden rounded-md border bg-card text-card-foreground shadow-sm", borderClass)} title={`${op.name} (${displayProfession})`}>
                                        <Image src={imageUrl} alt={op.name} fill sizes="(max-width: 640px) 20vw, 10vw" className={cn("object-cover")} unoptimized />
                                        <div className={`absolute inset-x-0 bottom-0 bg-black/60 px-1 py-0.5 text-center text-[10px] font-medium ${displayRarityColor} truncate`}>{op.name}</div>
                                    </div>
                                );
                            })}
                        </div>
                    ) : (
                        <p className="italic text-gray-500">Click &quot;Randomize Squad&quot; to generate a squad.</p>
                    )}
                </div>
            </div>

            {/* Bottom Section: Filters and Operator List */}
            <div className="flex min-h-0 flex-grow flex-col border-t pt-4">
                <h2 className="mb-3 flex-shrink-0 text-xl font-semibold">Filter & Select Operators</h2>

                {/* Filter Controls Row */}
                <div className="mb-1 flex flex-shrink-0 flex-wrap items-center gap-2">
                    <Input type="text" placeholder="Search operators..." value={searchTerm} onChange={(e) => setSearchTerm(e.target.value)} className="h-9 max-w-xs" />

                    {/* Rarity Filter Popover */}
                    <Popover open={rarityPopoverOpen} onOpenChange={setRarityPopoverOpen}>
                        <PopoverTrigger asChild>
                            <Button variant="outline" size="sm" className="w-[150px] justify-start">
                                <StarIcon className="mr-2 h-4 w-4" />
                                Rarity
                                {filterRarityNumeric.size > 0 && filterRarityNumeric.size < 6 && (
                                    <Badge variant="secondary" className="ml-auto">
                                        {filterRarityNumeric.size}
                                    </Badge>
                                )}
                            </Button>
                        </PopoverTrigger>
                        <PopoverContent className="w-[150px] p-0" align="start">
                            <Command>
                                <CommandList>
                                    <CommandEmpty>No results found.</CommandEmpty>
                                    <CommandGroup>
                                        {Array.from({ length: 6 }, (_, i) => i + 1).map((rarityNum) => {
                                            const isSelected = filterRarityNumeric.has(rarityNum);
                                            const rarityEnumEntry = Object.entries(OperatorRarity)
                                                .filter(([key]) => isNaN(Number(key)))
                                                .find(([, val]) => rarityToNumber(val as OperatorRarity) === rarityNum);
                                            const rarityEnum = rarityEnumEntry ? OperatorRarity[rarityEnumEntry[0] as keyof typeof OperatorRarity] : undefined;

                                            return (
                                                <CommandItem key={rarityNum} onSelect={() => handleRarityChange(rarityNum)} className="text-sm">
                                                    <div className={cn("mr-2 flex h-4 w-4 items-center justify-center rounded-sm border border-primary", isSelected ? "bg-primary text-primary-foreground" : "opacity-50 [&_svg]:invisible")}>
                                                        <CheckIcon className={cn("h-4 w-4")} />
                                                    </div>
                                                    <span className={`${rarityEnum ? RARITY_COLORS[rarityEnum] : "text-white"}`}>{rarityNum}★</span>
                                                </CommandItem>
                                            );
                                        })}
                                    </CommandGroup>
                                    {filterRarityNumeric.size > 0 && filterRarityNumeric.size < 6 && (
                                        <>
                                            <CommandSeparator />
                                            <CommandGroup>
                                                <CommandItem onSelect={() => setFilterRarityNumeric(new Set([1, 2, 3, 4, 5, 6]))} className="justify-center text-center text-xs text-muted-foreground">
                                                    Clear filters
                                                </CommandItem>
                                            </CommandGroup>
                                        </>
                                    )}
                                </CommandList>
                            </Command>
                        </PopoverContent>
                    </Popover>

                    {/* Profession Filter Popover */}
                    <Popover open={professionPopoverOpen} onOpenChange={setProfessionPopoverOpen}>
                        <PopoverTrigger asChild>
                            <Button variant="outline" size="sm" className="w-[150px] justify-start">
                                <ListIcon className="mr-2 h-4 w-4" /> {/* Use a relevant icon */}
                                Profession
                                {filterProfession.size > 0 && filterProfession.size < Object.keys(PROFESSION_MAP).length && (
                                    <Badge variant="secondary" className="ml-auto">
                                        {filterProfession.size}
                                    </Badge>
                                )}
                            </Button>
                        </PopoverTrigger>
                        <PopoverContent className="w-[200px] p-0" align="start">
                            <Command>
                                <CommandInput placeholder="Filter profession..." />
                                <CommandList>
                                    <CommandEmpty>No results found.</CommandEmpty>
                                    <CommandGroup>
                                        {Object.values(OperatorProfession).map((profEnumKey) => {
                                            const isSelected = filterProfession.has(profEnumKey);
                                            return (
                                                <CommandItem key={profEnumKey} onSelect={() => handleProfessionChange(profEnumKey)} className="text-sm">
                                                    <div className={cn("mr-2 flex h-4 w-4 items-center justify-center rounded-sm border border-primary", isSelected ? "bg-primary text-primary-foreground" : "opacity-50 [&_svg]:invisible")}>
                                                        <CheckIcon className={cn("h-4 w-4")} />
                                                    </div>
                                                    <span>{formatProfession(profEnumKey)}</span>
                                                </CommandItem>
                                            );
                                        })}
                                    </CommandGroup>
                                    {filterProfession.size > 0 && filterProfession.size < Object.keys(PROFESSION_MAP).length && (
                                        <>
                                            <CommandSeparator />
                                            <CommandGroup>
                                                <CommandItem onSelect={() => setFilterProfession(new Set())} className="justify-center text-center text-xs text-muted-foreground">
                                                    Clear filters
                                                </CommandItem>
                                            </CommandGroup>
                                        </>
                                    )}
                                </CommandList>
                            </Command>
                        </PopoverContent>
                    </Popover>

                    {/* Tag Filter Popover */}
                    <Popover open={tagPopoverOpen} onOpenChange={setTagPopoverOpen}>
                        <PopoverTrigger asChild>
                            <Button variant="outline" size="sm" className="w-[150px] justify-start">
                                <PlusCircleIcon className="mr-2 h-4 w-4" />
                                Tags
                                {selectedTags.size > 0 && (
                                    <Badge variant="secondary" className="ml-auto">
                                        {selectedTags.size}
                                    </Badge>
                                )}
                            </Button>
                        </PopoverTrigger>
                        <PopoverContent className="w-[200px] p-0" align="start">
                            <Command>
                                <CommandInput placeholder="Filter tags..." />
                                <CommandList>
                                    <CommandEmpty>No tags found.</CommandEmpty>
                                    <CommandGroup>
                                        {Array.from(allTags)
                                            .sort()
                                            .map((tag) => {
                                                const isSelected = selectedTags.has(tag);
                                                return (
                                                    <CommandItem key={tag} onSelect={() => handleTagSelect(tag)} className="text-sm">
                                                        <div className={cn("mr-2 flex h-4 w-4 items-center justify-center rounded-sm border border-primary", isSelected ? "bg-primary text-primary-foreground" : "opacity-50 [&_svg]:invisible")}>
                                                            <CheckIcon className={cn("h-4 w-4")} />
                                                        </div>
                                                        <span>{tag}</span>
                                                    </CommandItem>
                                                );
                                            })}
                                    </CommandGroup>
                                    {selectedTags.size > 0 && (
                                        <>
                                            <CommandSeparator />
                                            <CommandGroup>
                                                <CommandItem onSelect={() => setSelectedTags(new Set())} className="justify-center text-center text-xs text-muted-foreground">
                                                    Clear filters
                                                </CommandItem>
                                            </CommandGroup>
                                        </>
                                    )}
                                </CommandList>
                            </Command>
                        </PopoverContent>
                    </Popover>

                    {/* View Toggle Buttons */}
                    <div className="ml-auto flex items-center space-x-1">
                        <Button variant={viewMode === "list" ? "secondary" : "outline"} size="icon" onClick={() => setViewMode("list")} title="List View">
                            <ListIcon className="h-4 w-4" />
                        </Button>
                        <Button variant={viewMode === "grid" ? "secondary" : "outline"} size="icon" onClick={() => setViewMode("grid")} title="Grid View">
                            <GridIcon className="h-4 w-4" />
                        </Button>
                    </div>
                </div>
                {/* Display selected filters (excluding tags handled below) */}
                <div className="mb-2 flex flex-shrink-0 flex-wrap items-center gap-1">
                    {/* Selected Rarity */}
                    {Array.from(filterRarityNumeric)
                        .sort()
                        .filter((num) => num >= 1 && num <= 6)
                        .map((num) => (
                            <Badge key={`rarity-sel-${num}`} variant="secondary" className="cursor-pointer" onClick={() => handleRarityChange(num)}>
                                {num}★ &times;
                            </Badge>
                        ))}
                    {/* Selected Profession */}
                    {Array.from(filterProfession)
                        .sort((a, b) => formatProfession(a).localeCompare(formatProfession(b)))
                        .map((prof) => (
                            <Badge key={`prof-sel-${prof}`} variant="secondary" className="cursor-pointer" onClick={() => handleProfessionChange(prof)}>
                                {formatProfession(prof)} &times;
                            </Badge>
                        ))}
                    {/* Selected Tags */}
                    {Array.from(selectedTags)
                        .sort()
                        .map((tag) => (
                            <Badge key={`tag-sel-${tag}`} variant="secondary" className="cursor-pointer" onClick={() => handleTagSelect(tag)}>
                                {tag} &times;
                            </Badge>
                        ))}
                    {/* Clear Buttons Container - Removed ml-auto for testing */}
                    {filterRarityNumeric.size < 6 || filterProfession.size < Object.values(OperatorProfession).length || selectedTags.size > 0 || excludedOperators.size > 0 ? (
                        <div className="flex items-center gap-2">
                            {" "}
                            {/* Removed ml-auto */}
                            {/* Clear Excluded Button */}
                            {excludedOperators.size > 0 && (
                                <Button variant="link" size="sm" className="h-auto p-0 px-2 text-xs text-destructive hover:text-destructive/80" onClick={() => setExcludedOperators(new Set())}>
                                    Clear excluded ({excludedOperators.size})
                                </Button>
                            )}
                            {/* Clear Filters Button */}
                            {filterRarityNumeric.size < 6 || filterProfession.size < Object.values(OperatorProfession).length || selectedTags.size > 0 ? (
                                <Button
                                    variant="link"
                                    size="sm"
                                    className="h-auto p-0 px-2 text-xs text-muted-foreground"
                                    onClick={() => {
                                        setFilterRarityNumeric(new Set([1, 2, 3, 4, 5, 6]));
                                        setFilterProfession(new Set(Object.values(OperatorProfession)));
                                        setSelectedTags(new Set());
                                    }}
                                >
                                    Clear filters
                                </Button>
                            ) : null}
                        </div>
                    ) : null}
                </div>

                {/* Operator List/Grid Area */}
                <ScrollArea className="h-full min-h-[300px] flex-grow rounded-md border">
                    <div className="p-2">{isLoading ? <p className="text-center text-muted-foreground">Loading operators...</p> : filteredOperators.length > 0 ? viewMode === "list" ? <div className="space-y-0.5">{filteredOperators.map(renderOperatorListItem)}</div> : <div className="grid grid-cols-3 gap-2 sm:grid-cols-4 md:grid-cols-6 lg:grid-cols-7 xl:grid-cols-8">{filteredOperators.map(renderOperatorGridItem)}</div> : <p className="text-center text-muted-foreground">No operators match filters.</p>}</div>
                </ScrollArea>
                <p className="mt-2 flex-shrink-0 text-sm text-muted-foreground">
                    Showing {filteredOperators.length} / {allOperators.length} operators.
                    {excludedOperators.size > 0 && ` ${excludedOperators.size} excluded.`}
                </p>
            </div>
        </div>
    );
};

export default SquadRandomizer;
