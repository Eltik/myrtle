"use client";

import { ChevronsUpDown } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { DropdownMenu, DropdownMenuContent, DropdownMenuRadioGroup, DropdownMenuRadioItem, DropdownMenuSub, DropdownMenuSubContent, DropdownMenuSubTrigger, DropdownMenuTrigger } from "~/components/ui/dropdown-menu";
import { Button } from "./ui/button";

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

    const findDefaultOption = useCallback((items: Option[]): string | null => {
        for (const item of items) {
            if (item.default) {
                return item.value;
            }
            if (item.subOptions) {
                const defaultSubOption = findDefaultOption(item.subOptions);
                if (defaultSubOption) {
                    return defaultSubOption;
                }
            }
        }
        return null;
    }, []);

    useEffect(() => {
        if (!selectedOption) {
            const defaultOption = findDefaultOption(options);
            if (defaultOption) {
                setSelectedOption(defaultOption);
                onSelect(defaultOption);
            }
        }
    }, [options, onSelect, selectedOption, findDefaultOption]);

    const handleSelect = useCallback(
        (value: string) => {
            setSelectedOption(value);
            onSelect(value);
        },
        [onSelect],
    );

    const findSelectedLabel = useCallback((items: Option[], value: string): string | null => {
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
    }, []);

    const renderOptions = useCallback(
        (items: Option[]) => {
            return items.map((item) => {
                if (item.subOptions) {
                    return (
                        <DropdownMenuSub key={item.value}>
                            <DropdownMenuSubTrigger>
                                <span>{item.label}</span>
                            </DropdownMenuSubTrigger>
                            <DropdownMenuSubContent>
                                <DropdownMenuRadioGroup onValueChange={handleSelect} value={selectedOption ?? ""}>
                                    {renderOptions(item.subOptions)}
                                </DropdownMenuRadioGroup>
                            </DropdownMenuSubContent>
                        </DropdownMenuSub>
                    );
                } else {
                    return (
                        <DropdownMenuRadioItem className="group pl-2" key={item.value} value={item.value}>
                            <span className="group-aria-checked:pl-4">{item.label}</span>
                        </DropdownMenuRadioItem>
                    );
                }
            });
        },
        [selectedOption, handleSelect],
    );

    const selectedLabel = selectedOption ? findSelectedLabel(options, selectedOption) : null;

    return (
        <DropdownMenu>
            <DropdownMenuTrigger asChild>
                <Button className="w-[200px] justify-between" variant="outline">
                    {selectedLabel ?? placeholder}
                    <ChevronsUpDown className="ml-2 h-4 w-4 opacity-50" />
                </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent className="w-[200px]">
                <DropdownMenuRadioGroup onValueChange={handleSelect} value={selectedOption ?? ""}>
                    {renderOptions(options)}
                </DropdownMenuRadioGroup>
            </DropdownMenuContent>
        </DropdownMenu>
    );
}
