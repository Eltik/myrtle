import { ChevronDown } from "lucide-react";
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

    return (
        <div className="space-y-3">
            <span className="font-medium text-muted-foreground text-sm">{label}</span>
            <DropdownMenu>
                <DropdownMenuTrigger asChild>
                    <Button className="w-full justify-between" variant="outline">
                        <span className="truncate">{displayValue}</span>
                        <ChevronDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
                    </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent className="z-110 max-h-64 w-56 overflow-y-auto">
                    <DropdownMenuLabel>{label}</DropdownMenuLabel>
                    <DropdownMenuSeparator />
                    {options.map((option) => (
                        <DropdownMenuCheckboxItem checked={selectedOptions.includes(option)} key={option} onCheckedChange={() => onToggle(option)} onSelect={(e) => e.preventDefault()}>
                            {formatOption(option)}
                        </DropdownMenuCheckboxItem>
                    ))}
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
