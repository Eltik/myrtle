import { CheckIcon, PlusCircleIcon } from "lucide-react";
import { type Dispatch, type SetStateAction, useState } from "react";
import { Badge } from "~/components/ui/badge";
import { Button } from "~/components/ui/button";
import { Command, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList, CommandSeparator } from "~/components/ui/command";
import { Popover, PopoverContent, PopoverTrigger } from "~/components/ui/popover";
import { cn } from "~/lib/utils";

export const TagFilter = ({ selectedTags, setSelectedTags, allTags }: { selectedTags: Set<string>; setSelectedTags: Dispatch<SetStateAction<Set<string>>>; allTags: Set<string> }) => {
    const [tagPopoverOpen, setTagPopoverOpen] = useState(false);

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

    return (
        <Popover onOpenChange={setTagPopoverOpen} open={tagPopoverOpen}>
            <PopoverTrigger asChild>
                <Button className="w-[150px] justify-start" size="sm" variant="outline">
                    <PlusCircleIcon className="mr-2 h-4 w-4" />
                    Tags
                    {selectedTags.size > 0 && (
                        <Badge className="ml-auto" variant="secondary">
                            {selectedTags.size}
                        </Badge>
                    )}
                </Button>
            </PopoverTrigger>
            <PopoverContent align="start" className="w-[200px] p-0">
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
                                        <CommandItem className="text-sm" key={tag} onSelect={() => handleTagSelect(tag)}>
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
                                    <CommandItem className="justify-center text-center text-muted-foreground text-xs" onSelect={() => setSelectedTags(new Set())}>
                                        Clear filters
                                    </CommandItem>
                                </CommandGroup>
                            </>
                        )}
                    </CommandList>
                </Command>
            </PopoverContent>
        </Popover>
    );
};
