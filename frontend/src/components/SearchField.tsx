import { SearchIcon, X } from "lucide-react";
import type * as React from "react";
import { Button } from "#/components/ui/button";
import { InputGroup, InputGroupAddon, InputGroupInput } from "#/components/ui/input-group";

interface ISearchFieldProps {
    value: string;
    onChange: (value: string) => void;
    placeholder: string;
    /** The input's accessible label. */
    label: string;
    className?: string;
    inputRef?: React.Ref<HTMLInputElement>;
    /** When given, a clear button with this label shows while there is a query. */
    clearLabel?: string;
}

/** A search box: the magnifier, then the input. The background gallery's searches and the admin panel's are this. */
export function SearchField({ value, onChange, placeholder, label, className, inputRef, clearLabel }: ISearchFieldProps) {
    return (
        <InputGroup className={className}>
            <InputGroupAddon>
                <SearchIcon aria-hidden="true" />
            </InputGroupAddon>
            <InputGroupInput ref={inputRef} value={value} onChange={(e) => onChange((e.target as HTMLInputElement).value)} placeholder={placeholder} type="search" aria-label={label} />
            {clearLabel && value ? (
                <InputGroupAddon align="inline-end">
                    <Button variant="ghost" size="icon-xs" onClick={() => onChange("")} aria-label={clearLabel}>
                        <X aria-hidden="true" />
                    </Button>
                </InputGroupAddon>
            ) : null}
        </InputGroup>
    );
}
