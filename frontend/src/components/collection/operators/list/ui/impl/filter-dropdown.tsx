import { ChevronDown, Search } from "lucide-react";
import { useMemo, useState } from "react";
import { Button } from "~/components/ui/shadcn/button";
import { DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger } from "~/components/ui/shadcn/dropdown-menu";
import { FilterTag } from "./filter-tag";

interface FilterDropdownProps<T extends string> {
    label: string;
    placeholder: string;
    options: T[];
    selectedOptions: T[];
    onToggle: (option: T) => void;
    onRemove: (option: T) => void;
    formatOption?: (option: T) => string;
    formatSelected?: (option: T) => string;
}

export function FilterDropdown<T extends string>({ label, placeholder, options, selectedOptions, onToggle, onRemove, formatOption = (option) => option, formatSelected = formatOption }: FilterDropdownProps<T>) {
    const displayValue = selectedOptions.length > 0 ? selectedOptions.map(formatSelected).join(", ") : placeholder;
    const [search, setSearch] = useState("");

    const filteredOptions = useMemo(() => {
        if (!search.trim()) return options;
        const query = search.trim().toLowerCase();
        return options.filter((option) => formatOption(option).toLowerCase().includes(query));
    }, [search, options, formatOption]);

    return (
        <div className="space-y-3">
            <span className="font-medium text-muted-foreground text-sm">{label}</span>
            <DropdownMenu onOpenChange={() => setSearch("")}>
                <DropdownMenuTrigger asChild>
                    <Button className="w-full justify-between" variant="outline">
                        <span className="truncate">{displayValue}</span>
                        <ChevronDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
                    </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent className="z-110 max-h-64 w-56 overflow-y-auto">
                    <DropdownMenuLabel>{label}</DropdownMenuLabel>
                    <DropdownMenuSeparator />
                    <div className="sticky top-0 z-10 bg-popover px-2 pb-2">
                        <div className="relative">
                            <Search className="pointer-events-none absolute top-1/2 left-2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
                            <input
                                className="h-8 w-full rounded-md border border-input bg-transparent pr-2 pl-7 text-sm placeholder:text-muted-foreground focus:outline-none"
                                onChange={(e) => setSearch(e.target.value)}
                                onKeyDown={(e) => e.stopPropagation()}
                                placeholder={`Search ${label.toLowerCase()}...`}
                                type="text"
                                value={search}
                            />
                        </div>
                    </div>
                    {filteredOptions.length > 0 ? (
                        filteredOptions.map((option) => (
                            <DropdownMenuCheckboxItem checked={selectedOptions.includes(option)} key={option} onCheckedChange={() => onToggle(option)} onSelect={(e) => e.preventDefault()}>
                                {formatOption(option)}
                            </DropdownMenuCheckboxItem>
                        ))
                    ) : (
                        <div className="px-2 py-4 text-center text-muted-foreground text-sm">No results found</div>
                    )}
                </DropdownMenuContent>
            </DropdownMenu>
            {selectedOptions.length > 0 && (
                <div className="flex flex-wrap gap-1">
                    {selectedOptions.map((option) => (
                        <FilterTag key={option} label={formatSelected(option)} onRemove={() => onRemove(option)} />
                    ))}
                </div>
            )}
        </div>
    );
}
