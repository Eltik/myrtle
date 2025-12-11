import { CommandSeparator } from "~/components/ui/command";
import { CommandItem } from "~/components/ui/command";
import { CommandGroup } from "~/components/ui/command";
import { CommandEmpty } from "~/components/ui/command";
import { CommandInput, CommandList } from "~/components/ui/command";
import { PopoverContent } from "~/components/ui/popover";
import { PopoverTrigger } from "~/components/ui/popover";
import { Button } from "~/components/ui/button";
import { Command } from "~/components/ui/command";
import { Popover } from "~/components/ui/popover";
import { CheckIcon, ListIcon } from "lucide-react";
import { Badge } from "~/components/ui/badge";
import { PROFESSION_MAP } from "./helper";
import { OperatorProfession } from "~/types/impl/api/static/operator";
import { cn } from "~/lib/utils";
import { formatProfession } from "~/helper";
import { useState } from "react";
import type { Dispatch, SetStateAction } from "react";

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
        <>
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
        </>
    );
};
