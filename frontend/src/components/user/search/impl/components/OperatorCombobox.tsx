import { Search } from "lucide-react";
import { useCallback, useMemo } from "react";
import { Combobox, ComboboxChip, ComboboxChips, ComboboxChipsInput, ComboboxEmpty, ComboboxInput, ComboboxItem, ComboboxList, ComboboxPopup, ComboboxValue } from "#/components/ui/combobox";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { professionLabel } from "#/lib/registry/operator-display";
import { scoreMatch } from "#/lib/search/fuzzy";
import type { IOperatorIndexEntry } from "#/types/operators";
import { MAX_HAS } from "../searchControls";
import type { messages } from "./OperatorCombobox.messages";

interface IBaseProps {
    /** Obtainable operators, rarity first; undefined while the index loads. */
    operators: IOperatorIndexEntry[] | undefined;
    id: string;
    label: string;
    placeholder: string;
    className?: string;
}

const filter = (entry: IOperatorIndexEntry, query: string) => {
    if (!query.trim()) return true;
    return scoreMatch(query, { name: entry.name, extra: `${entry.appellation} ${professionLabel(entry.profession)} ${entry.subProfessionId} ${entry.rarity}` }) > 0;
};

const itemToStringLabel = (e: IOperatorIndexEntry) => e.name;
const itemToStringValue = (e: IOperatorIndexEntry) => e.id;
const isItemEqualToValue = (a: IOperatorIndexEntry, b: IOperatorIndexEntry) => a.id === b.id;

function useOperatorsById(operators: IOperatorIndexEntry[] | undefined): Map<string, IOperatorIndexEntry> {
    return useMemo(() => {
        const map = new Map<string, IOperatorIndexEntry>();
        for (const op of operators ?? []) map.set(op.id, op);
        return map;
    }, [operators]);
}

/** Several operators as chips inside the input; the value is their ids. */
export function OperatorMultiCombobox({ operators, value, onChange, id, label, placeholder, className }: IBaseProps & { value: string[]; onChange: (ids: string[]) => void }) {
    const t: TypedT<typeof messages> = useT("user");
    const byId = useOperatorsById(operators);
    // An id the index has not resolved yet (still loading) is kept out of the
    // chips but not out of the URL: a change only writes what the list knows.
    const selected = useMemo(() => value.map((v) => byId.get(v)).filter((e): e is IOperatorIndexEntry => e !== undefined), [value, byId]);
    const handleChange = useCallback((next: IOperatorIndexEntry[]) => onChange(next.slice(0, MAX_HAS).map((e) => e.id)), [onChange]);

    return (
        <Combobox<IOperatorIndexEntry, true> multiple items={operators ?? []} value={selected} onValueChange={handleChange} filter={filter} itemToStringLabel={itemToStringLabel} itemToStringValue={itemToStringValue} isItemEqualToValue={isItemEqualToValue}>
            <ComboboxChips className={className} startAddon={<Search />}>
                <ComboboxValue>
                    {(chips: IOperatorIndexEntry[]) => (
                        <>
                            {chips.map((e) => (
                                <ComboboxChip key={e.id} aria-label={e.name}>
                                    {e.name}
                                </ComboboxChip>
                            ))}
                            <ComboboxChipsInput id={id} aria-label={label} placeholder={operators === undefined ? t("search.operatorPicker.loading") : chips.length === 0 ? placeholder : ""} />
                        </>
                    )}
                </ComboboxValue>
            </ComboboxChips>
            <OperatorPopup />
        </Combobox>
    );
}

/** One operator; the value is its id, the empty string for none. */
export function OperatorSingleCombobox({ operators, value, onChange, id, label, placeholder, className }: IBaseProps & { value: string; onChange: (id: string) => void }) {
    const t: TypedT<typeof messages> = useT("user");
    const byId = useOperatorsById(operators);
    const selected = (value && byId.get(value)) || null;
    const handleChange = useCallback((next: IOperatorIndexEntry | null) => onChange(next?.id ?? ""), [onChange]);

    return (
        <Combobox<IOperatorIndexEntry, false> items={operators ?? []} value={selected} onValueChange={handleChange} filter={filter} itemToStringLabel={itemToStringLabel} itemToStringValue={itemToStringValue} isItemEqualToValue={isItemEqualToValue}>
            <ComboboxInput id={id} aria-label={label} placeholder={operators === undefined ? t("search.operatorPicker.loading") : placeholder} startAddon={<Search />} showClear className={className} />
            <OperatorPopup />
        </Combobox>
    );
}

function OperatorPopup() {
    const t: TypedT<typeof messages> = useT("user");
    return (
        <ComboboxPopup className="w-[min(360px,calc(100vw-2rem))]">
            <ComboboxEmpty>{t("search.operatorPicker.empty")}</ComboboxEmpty>
            <ComboboxList>
                {(op: IOperatorIndexEntry) => (
                    <ComboboxItem key={op.id} value={op}>
                        <span className="flex items-center gap-2">
                            <span aria-hidden="true" className="inline-flex size-7 shrink-0 items-center justify-center overflow-hidden rounded-md bg-muted">
                                <OperatorAvatar charId={op.id} name={op.name} />
                            </span>
                            <span className="flex min-w-0 flex-1 flex-col">
                                <span className="truncate font-medium text-[13px]">{op.name}</span>
                                <span className="truncate text-[10.5px] text-muted-foreground">
                                    {t("search.operatorPicker.rarity", { rarity: op.rarity })} {professionLabel(op.profession)}
                                </span>
                            </span>
                        </span>
                    </ComboboxItem>
                )}
            </ComboboxList>
        </ComboboxPopup>
    );
}
