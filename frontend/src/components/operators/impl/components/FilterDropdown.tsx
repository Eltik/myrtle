import { Fragment, useId, useMemo } from "react";
import { Combobox, ComboboxCollection, ComboboxEmpty, ComboboxGroup, ComboboxGroupLabel, ComboboxInput, ComboboxItem, ComboboxList, ComboboxPopup, ComboboxSeparator } from "#/components/ui/combobox";

interface Group {
    key: string;
    label: string;
    items: string[];
}

interface FilterDropdownProps {
    label: string;
    placeholder: string;
    options: string[];
    selected: string[];
    onChange: (values: string[]) => void;
    /** Pretty-print an option (e.g. lowercase id → display name). */
    formatOption?: (value: string) => string;
    /** When provided, items are bucketed by the returned group key and rendered with headers + dividers. */
    groupBy?: (value: string) => string;
    /** Preferred order for group keys. Unknown keys fall to the end (alphabetical by label). */
    groupOrder?: readonly string[];
    /** Pretty-print a group key for its header label. */
    formatGroup?: (key: string) => string;
}

export function FilterDropdown({ label, placeholder, options, selected, onChange, formatOption, groupBy, groupOrder, formatGroup }: FilterDropdownProps) {
    const id = useId();
    const format = formatOption ?? ((v: string) => v);

    const groups = useMemo<Group[] | null>(() => {
        if (!groupBy) return null;
        const buckets = new Map<string, string[]>();
        for (const opt of options) {
            const key = groupBy(opt);
            const bucket = buckets.get(key);
            if (bucket) bucket.push(opt);
            else buckets.set(key, [opt]);
        }
        const orderIdx = (k: string) => {
            const idx = groupOrder?.indexOf(k) ?? -1;
            return idx === -1 ? Number.MAX_SAFE_INTEGER : idx;
        };
        const labelFor = formatGroup ?? ((k: string) => k);
        return Array.from(buckets.entries())
            .map(
                ([key, items]): Group => ({
                    key,
                    label: labelFor(key),
                    items: [...items].sort((a, b) => format(a).localeCompare(format(b))),
                }),
            )
            .sort((a, b) => {
                const oa = orderIdx(a.key);
                const ob = orderIdx(b.key);
                if (oa !== ob) return oa - ob;
                return a.label.localeCompare(b.label);
            });
    }, [options, groupBy, groupOrder, formatGroup, format]);

    const items = groups ?? options;

    return (
        <div className="space-y-2">
            <label htmlFor={id} className="block text-[12px] font-medium leading-none text-muted-foreground">
                {label}
            </label>
            <Combobox<string, true> multiple items={items} value={selected} onValueChange={onChange} itemToStringLabel={format}>
                <ComboboxInput id={id} placeholder={selected.length > 0 ? `${selected.length} selected` : placeholder} />
                <ComboboxPopup>
                    <ComboboxEmpty>No matches</ComboboxEmpty>
                    <ComboboxList>
                        {groups
                            ? (group: Group) => (
                                  <Fragment key={group.key}>
                                      <ComboboxGroup items={group.items}>
                                          <ComboboxGroupLabel>{group.label}</ComboboxGroupLabel>
                                          <ComboboxCollection>
                                              {(option: string) => (
                                                  <ComboboxItem key={option} value={option}>
                                                      {format(option)}
                                                  </ComboboxItem>
                                              )}
                                          </ComboboxCollection>
                                      </ComboboxGroup>
                                      <ComboboxSeparator />
                                  </Fragment>
                              )
                            : (option: string) => (
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
