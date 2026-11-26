import { useState } from "react";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { Combobox, ComboboxEmpty, ComboboxInput, ComboboxItem, ComboboxList, ComboboxPopup, ComboboxStatus } from "#/components/ui/combobox";
import { getAvatarById } from "#/lib/utils";

/** One account a {@link PersonCombobox} can pick. */
export interface IPersonOption {
    /** The account id (UUID). */
    id: string;
    name: string;
    /** A second, dimmer line: a uid, a server, a count. */
    hint?: string | null;
    avatarId?: string | null;
}

interface IPersonComboboxProps {
    value: IPersonOption | null;
    onChange: (next: IPersonOption | null) => void;
    /** Results for the current query; the caller searches, the box does not filter. */
    options: readonly IPersonOption[];
    /** Every keystroke. Debounce before searching. */
    onQueryChange: (query: string) => void;
    loading?: boolean;
    label: string;
    placeholder: string;
    emptyText: string;
    loadingText?: string;
    className?: string;
}

/**
 * Picks one account from results the caller fetches. Single-select, with a
 * clear button once someone is picked. The caller owns the search (an
 * endpoint, a local list ranked through `fuzzy.ts`, or both), so the box sets
 * `filter={null}` and shows `options` as given.
 */
export function PersonCombobox({ value, onChange, options, onQueryChange, loading, label, placeholder, emptyText, loadingText, className }: IPersonComboboxProps): React.ReactElement {
    // The picked person stays listed while a new query loads, so the input's label never blanks.
    const [open, setOpen] = useState(false);
    const items = value && !options.some((o) => o.id === value.id) ? [value, ...options] : [...options];
    return (
        <Combobox<IPersonOption>
            items={items}
            filter={null}
            value={value}
            onValueChange={(next) => onChange(next ?? null)}
            onInputValueChange={(query, details) => {
                // Selecting an item writes its label into the input; that is not a search.
                if (details.reason === "input-change") onQueryChange(query);
                else if (details.reason === "input-clear" || details.reason === "clear-press") onQueryChange("");
            }}
            onOpenChange={setOpen}
            itemToStringLabel={(o) => o.name}
            itemToStringValue={(o) => o.id}
            isItemEqualToValue={(a, b) => a.id === b.id}
        >
            <ComboboxInput size="sm" aria-label={label} placeholder={placeholder} showClear={value != null} className={className} />
            <ComboboxPopup className="w-[min(320px,calc(100vw-2rem))]">
                {open && loading ? <ComboboxStatus className="px-3 py-2 text-[13px] text-muted-foreground">{loadingText}</ComboboxStatus> : null}
                <ComboboxEmpty>{loading ? null : emptyText}</ComboboxEmpty>
                <ComboboxList>
                    {(o: IPersonOption) => (
                        <ComboboxItem key={o.id} value={o}>
                            <span className="flex min-w-0 flex-1 items-center gap-2">
                                <Avatar className="size-5 shrink-0 rounded-full border border-border bg-muted text-[9px]">
                                    {o.avatarId ? <AvatarImage src={getAvatarById(o.avatarId)} alt="" /> : null}
                                    <AvatarFallback>{o.name.charAt(0).toUpperCase()}</AvatarFallback>
                                </Avatar>
                                <span className="min-w-0 flex-1 truncate">{o.name}</span>
                                {o.hint ? <span className="shrink-0 font-mono text-[11px] text-muted-foreground tabular-nums">{o.hint}</span> : null}
                            </span>
                        </ComboboxItem>
                    )}
                </ComboboxList>
            </ComboboxPopup>
        </Combobox>
    );
}
