"use client";

import { ChevronsUpDown } from "lucide-react";

import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger, DropdownMenuSub, DropdownMenuSubContent, DropdownMenuSubTrigger, DropdownMenuRadioGroup, DropdownMenuRadioItem } from "~/components/ui/dropdown-menu";
import { Button } from "./ui/button";
import { useState } from "react";

interface Option {
    label: string;
    value: string;
    default?: boolean;
    subOptions?: Option[];
}

interface NestedDropdownProps {
    options: Option[];
    placeholder?: string;
    onSelect: (value: string) => void;
}

export function NestedDropdown({ options, placeholder = "Select an option", onSelect }: NestedDropdownProps) {
    const [selectedOption, setSelectedOption] = useState<string | null>(null);

    const handleSelect = (value: string) => {
        setSelectedOption(value);
        onSelect(value);
    };

    const findSelectedLabel = (items: Option[], value: string): string | null => {
        for (const item of items) {
            if (item.value === value) {
                return item.label;
            }
            if (item.subOptions) {
                const subLabel = findSelectedLabel(item.subOptions, value);
                if (subLabel) {
                    return subLabel;
                }
            }
        }
        return null;
    };

    const renderOptions = (items: Option[]) => {
        return items.map((item) => {
            if (item.subOptions) {
                return (
                    <DropdownMenuSub key={item.value}>
                        <DropdownMenuSubTrigger>
                            <span>{item.label}</span>
                        </DropdownMenuSubTrigger>
                        <DropdownMenuSubContent>
                            <DropdownMenuRadioGroup value={selectedOption ?? ""} onValueChange={handleSelect}>
                                {renderOptions(item.subOptions)}
                            </DropdownMenuRadioGroup>
                        </DropdownMenuSubContent>
                    </DropdownMenuSub>
                );
            } else {
                return (
                    <DropdownMenuRadioItem key={item.value} value={item.value} className="group pl-2">
                        <span className="group-aria-checked:pl-4">{item.label}</span>
                    </DropdownMenuRadioItem>
                );
            }
        });
    };

    const selectedLabel = selectedOption ? findSelectedLabel(options, selectedOption) : null;

    return (
        <DropdownMenu>
            <DropdownMenuTrigger asChild>
                <Button variant="outline" className="w-[200px] justify-between">
                    {selectedLabel ?? placeholder}
                    <ChevronsUpDown className="ml-2 h-4 w-4 opacity-50" />
                </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent className="w-[200px]">
                <DropdownMenuRadioGroup value={selectedOption ?? ""} onValueChange={handleSelect}>
                    {renderOptions(options)}
                </DropdownMenuRadioGroup>
            </DropdownMenuContent>
        </DropdownMenu>
    );
}