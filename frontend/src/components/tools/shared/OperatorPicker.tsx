import { useQuery } from "@tanstack/react-query";
import { Plus, Search } from "lucide-react";
import * as React from "react";
import { Combobox, ComboboxEmpty, ComboboxInput, ComboboxItem, ComboboxList, ComboboxPopup } from "#/components/ui/combobox";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { professionLabel } from "#/lib/registry/operator-display";
import { scoreMatch } from "#/lib/search/fuzzy";
import type { IOperatorIndexEntry } from "#/types/operators";
import type { IOperatorListEntry } from "./types";

interface IOperatorPickerProps {
    operators: IOperatorListEntry[];
    isLoading: boolean;
    isError: boolean;
    error?: unknown;
    existingCount: number;
    onAdd: (op: IOperatorListEntry) => void;
    /** Singular noun for labels/copy, e.g. "operator" or "healer". */
    noun: string;
}

interface IPickerEntry {
    op: IOperatorListEntry;
    index?: IOperatorIndexEntry;
}

export function OperatorPicker({ operators, isLoading, isError, error, existingCount, onAdd, noun }: IOperatorPickerProps): React.ReactElement {
    const { data: indexEntries } = useQuery(operatorsIndexQueryOptions());

    const indexById = React.useMemo(() => {
        const map = new Map<string, IOperatorIndexEntry>();
        for (const e of indexEntries ?? []) map.set(e.id, e);
        return map;
    }, [indexEntries]);

    const items = React.useMemo<IPickerEntry[]>(() => {
        const enriched = operators.map((op) => ({ op, index: indexById.get(op.id) }));
        return enriched.sort((a, b) => {
            const ra = a.index?.rarity ?? -1;
            const rb = b.index?.rarity ?? -1;
            if (rb !== ra) return rb - ra;
            return a.op.name.localeCompare(b.op.name);
        });
    }, [operators, indexById]);

    const handleChange = (value: IPickerEntry | null) => {
        if (value) onAdd(value.op);
    };

    const filter = React.useCallback((entry: IPickerEntry, query: string) => {
        if (!query.trim()) return true;
        const idx = entry.index;
        const target = {
            name: entry.op.name,
            extra: idx ? `${idx.appellation} ${professionLabel(idx.profession)} ${idx.subProfessionId} ${idx.rarity} ${(idx.tagList ?? []).join(" ")} ${idx.nationId}` : "",
        };
        return scoreMatch(query, target) > 0;
    }, []);

    return (
        <div className="space-y-2">
            <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="operator-picker">
                {existingCount === 0 ? `Add ${aOrAn(noun)}` : `Add another ${noun}`}
            </label>
            <Combobox<IPickerEntry, false> items={items} value={null} onValueChange={handleChange} filter={filter} itemToStringLabel={(e) => e.op.name} itemToStringValue={(e) => e.op.id}>
                <ComboboxInput id="operator-picker" placeholder={isLoading ? `Loading ${noun}s...` : `Search ${items.length} ${noun}${items.length === 1 ? "" : "s"}...`} startAddon={existingCount === 0 ? <Search /> : <Plus />} />
                <ComboboxPopup>
                    <ComboboxEmpty>No matching {noun}s.</ComboboxEmpty>
                    <ComboboxList>
                        {(entry: IPickerEntry) => {
                            const op = entry.op;
                            const idx = entry.index;
                            const rarity = idx?.rarity ?? 0;
                            return (
                                <ComboboxItem key={op.id} value={entry}>
                                    <span className="flex items-center gap-2">
                                        <span aria-hidden="true" className="inline-flex size-7 shrink-0 items-center justify-center overflow-hidden rounded-md bg-muted font-semibold text-[10px] text-muted-foreground">
                                            <OperatorAvatar charId={op.id} name={op.name} />
                                        </span>
                                        <span className="flex min-w-0 flex-1 flex-col">
                                            <span className="truncate font-medium text-[13px]">{op.name}</span>
                                            {idx && (
                                                <span className="truncate text-[10.5px] text-muted-foreground">
                                                    {rarity > 0 ? `${rarity}★ · ` : ""}
                                                    {professionLabel(idx.profession)}
                                                </span>
                                            )}
                                        </span>
                                    </span>
                                </ComboboxItem>
                            );
                        }}
                    </ComboboxList>
                </ComboboxPopup>
            </Combobox>
            {isError ? (
                <p className="rounded border border-destructive/30 bg-destructive/8 px-2 py-1.5 text-[11px] text-foreground/90 leading-snug">
                    Couldn't load {noun} list. {(error as Error)?.message ?? ""}
                </p>
            ) : existingCount === 0 ? (
                <p className="text-[11px] text-muted-foreground leading-snug">Pick any {noun} to start. You can add the same one multiple times to compare different builds.</p>
            ) : existingCount === 1 ? (
                <p className="text-[11px] text-muted-foreground leading-snug">Tip: add the same {noun} again with the picker, then tweak skill/level/module to compare two setups.</p>
            ) : null}
        </div>
    );
}

function aOrAn(noun: string): string {
    return /^[aeiou]/i.test(noun) ? `an ${noun}` : `a ${noun}`;
}
