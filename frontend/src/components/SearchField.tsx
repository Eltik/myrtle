import { SearchIcon } from "lucide-react";
import { InputGroup, InputGroupAddon, InputGroupInput } from "#/components/ui/input-group";

interface ISearchFieldProps {
    value: string;
    onChange: (value: string) => void;
    placeholder: string;
    /** The input's accessible label. */
    label: string;
    className?: string;
}

/** A search box: the magnifier, then the input. The background gallery's searches and the admin panel's are this. */
export function SearchField({ value, onChange, placeholder, label, className }: ISearchFieldProps) {
    return (
        <InputGroup className={className}>
            <InputGroupAddon>
                <SearchIcon aria-hidden="true" />
            </InputGroupAddon>
            <InputGroupInput value={value} onChange={(e) => onChange((e.target as HTMLInputElement).value)} placeholder={placeholder} type="search" aria-label={label} />
        </InputGroup>
    );
}
