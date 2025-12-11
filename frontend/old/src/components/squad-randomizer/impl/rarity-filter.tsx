import { CheckIcon, StarIcon } from "lucide-react";
import { type Dispatch, type SetStateAction, useState } from "react";
import { Badge } from "~/components/ui/badge";
import { Button } from "~/components/ui/button";
import { Command, CommandEmpty, CommandGroup, CommandItem, CommandList, CommandSeparator } from "~/components/ui/command";
import { Popover, PopoverContent, PopoverTrigger } from "~/components/ui/popover";
import { cn } from "~/lib/utils";

export const RarityFilter = ({ filterRarityNumeric, setFilterRarityNumeric }: { filterRarityNumeric: Set<number>; setFilterRarityNumeric: Dispatch<SetStateAction<Set<number>>> }) => {
    const [rarityPopoverOpen, setRarityPopoverOpen] = useState(false);

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

    return (
        <>
            {/* Rarity Filter Popover */}
            <Popover onOpenChange={setRarityPopoverOpen} open={rarityPopoverOpen}>
                <PopoverTrigger asChild>
                    <Button className="w-[150px] justify-start" size="sm" variant="outline">
                        <StarIcon className="mr-2 h-4 w-4" />
                        Rarity
                        {filterRarityNumeric.size > 0 && filterRarityNumeric.size < 6 && (
                            <Badge className="ml-auto" variant="secondary">
                                {filterRarityNumeric.size}
                            </Badge>
                        )}
                    </Button>
                </PopoverTrigger>
                <PopoverContent align="start" className="w-[150px] p-0">
                    <Command>
                        <CommandList>
                            <CommandEmpty>No results found.</CommandEmpty>
                            <CommandGroup>
                                {Array.from({ length: 6 }, (_, i) => i + 1).map((rarityNum) => {
                                    const isSelected = filterRarityNumeric.has(rarityNum);

                                    return (
                                        <CommandItem className="text-sm" key={rarityNum} onSelect={() => handleRarityChange(rarityNum)}>
                                            <div className={cn("mr-2 flex h-4 w-4 items-center justify-center rounded-sm border border-primary", isSelected ? "bg-primary text-primary-foreground" : "opacity-50 [&_svg]:invisible")}>
                                                <CheckIcon className={cn("h-4 w-4")} />
                                            </div>
                                            <span>{rarityNum}★</span>
                                        </CommandItem>
                                    );
                                })}
                            </CommandGroup>
                            {filterRarityNumeric.size > 0 && filterRarityNumeric.size < 6 && (
                                <>
                                    <CommandSeparator />
                                    <CommandGroup>
                                        <CommandItem className="justify-center text-center text-muted-foreground text-xs" onSelect={() => setFilterRarityNumeric(new Set([1, 2, 3, 4, 5, 6]))}>
                                            Clear filters
                                        </CommandItem>
                                    </CommandGroup>
                                </>
                            )}
                        </CommandList>
                    </Command>
                </PopoverContent>
            </Popover>
        </>
    );
};
