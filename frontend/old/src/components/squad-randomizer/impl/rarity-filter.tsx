import { CommandGroup, CommandItem, CommandSeparator } from "~/components/ui/command";
import { CommandEmpty } from "~/components/ui/command";
import { CheckIcon, StarIcon } from "lucide-react";
import { CommandList } from "~/components/ui/command";
import { type Dispatch, type SetStateAction, useState } from "react";
import { Badge } from "~/components/ui/badge";
import { Button } from "~/components/ui/button";
import { Command } from "~/components/ui/command";
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

                                    return (
                                        <CommandItem key={rarityNum} onSelect={() => handleRarityChange(rarityNum)} className="text-sm">
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
        </>
    );
};
