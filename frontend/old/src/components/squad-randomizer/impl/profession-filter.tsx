import { CheckIcon, ListIcon } from "lucide-react";
import type { Dispatch, SetStateAction } from "react";
import { useState } from "react";
import { Badge } from "~/components/ui/badge";
import { Button } from "~/components/ui/button";
import { Command, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList, CommandSeparator } from "~/components/ui/command";
import { Popover, PopoverContent, PopoverTrigger } from "~/components/ui/popover";
import { formatProfession } from "~/helper";
import { cn } from "~/lib/utils";
import { OperatorProfession } from "~/types/impl/api/static/operator";
import { PROFESSION_MAP } from "./helper";

export const ProfessionFilter = ({ filterProfession, setFilterProfession }: { filterProfession: Set<OperatorProfession>; setFilterProfession: Dispatch<SetStateAction<Set<OperatorProfession>>> }) => {
    const [professionPopoverOpen, setProfessionPopoverOpen] = useState(false);

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

    return (
        <Popover onOpenChange={setProfessionPopoverOpen} open={professionPopoverOpen}>
            <PopoverTrigger asChild>
                <Button className="w-[150px] justify-start" size="sm" variant="outline">
                    <ListIcon className="mr-2 h-4 w-4" /> {/* Use a relevant icon */}
                    Profession
                    {filterProfession.size > 0 && filterProfession.size < Object.keys(PROFESSION_MAP).length && (
                        <Badge className="ml-auto" variant="secondary">
                            {filterProfession.size}
                        </Badge>
                    )}
                </Button>
            </PopoverTrigger>
            <PopoverContent align="start" className="w-[200px] p-0">
                <Command>
                    <CommandInput placeholder="Filter profession..." />
                    <CommandList>
                        <CommandEmpty>No results found.</CommandEmpty>
                        <CommandGroup>
                            {Object.values(OperatorProfession).map((profEnumKey) => {
                                const isSelected = filterProfession.has(profEnumKey);
                                return (
                                    <CommandItem className="text-sm" key={profEnumKey} onSelect={() => handleProfessionChange(profEnumKey)}>
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
                                    <CommandItem className="justify-center text-center text-muted-foreground text-xs" onSelect={() => setFilterProfession(new Set())}>
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
