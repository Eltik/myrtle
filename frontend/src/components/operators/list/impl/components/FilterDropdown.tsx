import { X } from "lucide-react";
import { Fragment, type ReactNode, useId, useMemo } from "react";
import { Combobox, ComboboxCollection, ComboboxEmpty, ComboboxGroup, ComboboxGroupLabel, ComboboxInput, ComboboxItem, ComboboxList, ComboboxPopup, ComboboxSeparator } from "#/components/ui/combobox";

interface IGroup {
    key: string;
    label: string;
    items: string[];
}

interface IFilterDropdownProps {
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
    /** Render a leading icon for each option, placed before the formatted label. */
    renderOptionIcon?: (value: string) => ReactNode;
}

export function FilterDropdown({ label, placeholder, options, selected, onChange, formatOption, groupBy, groupOrder, formatGroup, renderOptionIcon }: IFilterDropdownProps) {
    const id = useId();
    const format = formatOption ?? ((v: string) => v);

    const groups = useMemo<IGroup[] | null>(() => {
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
                ([key, items]): IGroup => ({
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
            <label htmlFor={id} className="block font-medium text-[12px] text-muted-foreground leading-none">
                {label}
            </label>
            <Combobox<string, true> multiple items={items} value={selected} onValueChange={onChange} itemToStringLabel={format}>
                <ComboboxInput id={id} placeholder={selected.length > 0 ? `${selected.length} selected` : placeholder} />
                <ComboboxPopup>
                    <ComboboxEmpty>No matches</ComboboxEmpty>
                    <ComboboxList>
                        {groups
                            ? (group: IGroup) => (
                                  <Fragment key={group.key}>
                                      <ComboboxGroup items={group.items}>
                                          <ComboboxGroupLabel>{group.label}</ComboboxGroupLabel>
                                          <ComboboxCollection>
                                              {(option: string) => (
                                                  <ComboboxItem key={option} value={option}>
                                                      {renderOptionIcon ? (
                                                          <span className="flex items-center gap-2">
                                                              {renderOptionIcon(option)}
                                                              <span>{format(option)}</span>
                                                          </span>
                                                      ) : (
                                                          format(option)
                                                      )}
                                                  </ComboboxItem>
                                              )}
                                          </ComboboxCollection>
                                      </ComboboxGroup>
                                      <ComboboxSeparator />
                                  </Fragment>
                              )
                            : (option: string) => (
                                  <ComboboxItem key={option} value={option}>
                                      {renderOptionIcon ? (
                                          <span className="flex items-center gap-2">
                                              {renderOptionIcon(option)}
                                              <span>{format(option)}</span>
                                          </span>
                                      ) : (
                                          format(option)
                                      )}
                                  </ComboboxItem>
                              )}
                    </ComboboxList>
                </ComboboxPopup>
            </Combobox>
            {selected.length > 0 && (
                <div className="flex flex-wrap gap-1">
                    {selected.map((opt) => (
                        <FilterTag key={opt} label={format(opt)} onRemove={() => onChange(selected.filter((v) => v !== opt))} />
                    ))}
                </div>
            )}
        </div>
    );
}

function FilterTag({ label, onRemove }: { label: string; onRemove: () => void }) {
    return (
        <span className="inline-flex items-center gap-1 rounded-md bg-primary/20 px-2 py-0.5 text-foreground text-xs">
            {label}
            <button type="button" className="hover:text-destructive" onClick={onRemove}>
                <X className="h-3 w-3" />
            </button>
        </span>
    );
}
