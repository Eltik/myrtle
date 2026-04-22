import { useId } from "react";
import { Combobox, ComboboxEmpty, ComboboxInput, ComboboxItem, ComboboxList, ComboboxPopup } from "#/components/ui/combobox";

interface FilterDropdownProps {
    label: string;
    placeholder: string;
    options: string[];
    selected: string[];
    onChange: (values: string[]) => void;
    /** Pretty-print an option (e.g. lowercase id → display name). */
    formatOption?: (value: string) => string;
}

export function FilterDropdown({ label, placeholder, options, selected, onChange, formatOption }: FilterDropdownProps) {
    const id = useId();
    const format = formatOption ?? ((v: string) => v);

    return (
        <div className="space-y-2">
            <label htmlFor={id} className="block text-[12px] font-medium leading-none text-muted-foreground">
                {label}
            </label>
            <Combobox<string, true> multiple items={options} value={selected} onValueChange={onChange} itemToStringLabel={format}>
                <ComboboxInput id={id} placeholder={selected.length > 0 ? `${selected.length} selected` : placeholder} />
                <ComboboxPopup>
                    <ComboboxEmpty>No matches</ComboboxEmpty>
                    <ComboboxList>
                        {(option: string) => (
                            <ComboboxItem key={option} value={option}>
                                {format(option)}
                            </ComboboxItem>
                        )}
                    </ComboboxList>
                </ComboboxPopup>
            </Combobox>
        </div>
    );
}
