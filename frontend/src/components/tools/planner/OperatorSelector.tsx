import { useQuery } from "@tanstack/react-query";
import * as React from "react";

import { Combobox, ComboboxEmpty, ComboboxInput, ComboboxItem, ComboboxList, ComboboxPopup, ComboboxPrimitive } from "#/components/ui/combobox";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { useOperatorName } from "#/hooks/use-operator-name";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { upcomingQueryOptions } from "#/lib/api/upcoming";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { professionLabel } from "#/lib/registry/operator-display";
import { searchAndRank } from "#/lib/search/fuzzy";
import { cn } from "#/lib/utils";
import type { IOperatorIndexEntry } from "#/types/operators";
import type { messages as bulkMessages } from "./BulkPlanDialog.messages";
import type { messages } from "./OperatorPlannerDialog.messages";

/** One row of the operator picker. */
export interface IOperatorOption extends Pick<IOperatorIndexEntry, "id" | "name" | "appellation" | "rarity" | "profession" | "subProfessionId" | "tagList" | "nationId"> {
    /** `name` under the Latin-names preference; what the list shows and sorts by. */
    displayName: string;
    /** Not yet on the selected server; listed first, with the CN avatar. */
    isUpcoming: boolean;
}

function toOption(op: IOperatorIndexEntry, displayName: string, isUpcoming: boolean): IOperatorOption {
    return {
        id: op.id,
        name: op.name,
        displayName,
        appellation: op.appellation,
        rarity: op.rarity,
        profession: op.profession,
        subProfessionId: op.subProfessionId,
        tagList: op.tagList,
        nationId: op.nationId,
        isUpcoming,
    };
}

/** Upcoming operators first, then rarity high to low, then display name. */
function compareOptions(a: IOperatorOption, b: IOperatorOption): number {
    if (a.isUpcoming !== b.isUpcoming) return a.isUpcoming ? -1 : 1;
    if (b.rarity !== a.rarity) return b.rarity - a.rarity;
    return a.displayName.localeCompare(b.displayName);
}

/**
 * The picker's rows: every obtainable operator on the server plus the upcoming
 * ones, ranked against `searchQuery`. Called by the dialog itself rather than
 * the picker, so both lists start loading as soon as the dialog mounts.
 */
export function useOperatorOptions(server: string, searchQuery: string, selectedOperatorId: string | null) {
    const operatorName = useOperatorName();
    // The index, not the full table: the picker reads only names, rarity and
    // tags. The full table is ~40 MB decoded through a server fn, and a slow or
    // failed fetch of it left the picker on "Loading operators..." with only
    // the upcoming CN operators listed.
    const { data: operators = [], isLoading: isOperatorsLoading } = useQuery(operatorsIndexQueryOptions(server));
    const { data: upcoming = [], isLoading: isUpcomingLoading } = useQuery(upcomingQueryOptions(server));

    const sortedOptions = React.useMemo(() => {
        const upcomingOptions = upcoming.filter((op) => !op.isNotObtainable).map((op) => toOption(op, operatorName(op), true));
        const releasedOptions = operators.filter((op) => !op.isNotObtainable).map((op) => toOption(op, operatorName(op), false));
        return [...upcomingOptions, ...releasedOptions].sort(compareOptions);
    }, [operators, upcoming, operatorName]);

    const options = React.useMemo(() => {
        if (!searchQuery.trim()) return sortedOptions;
        const results = searchAndRank(searchQuery, sortedOptions, (op) => ({
            name: op.displayName,
            aliases: [op.name, op.appellation],
            extra: `${professionLabel(op.profession)} ${op.subProfessionId} ${op.rarity}★ ${(op.tagList ?? []).join(" ")} ${op.nationId}`,
        }));
        return results.map((r) => r.item);
    }, [searchQuery, sortedOptions]);

    const selectedOption = React.useMemo(() => {
        if (!selectedOperatorId) return null;
        return sortedOptions.find((item) => item.id === selectedOperatorId) || null;
    }, [selectedOperatorId, sortedOptions]);

    return { options, allOptions: sortedOptions, selectedOption, isLoading: isOperatorsLoading || isUpcomingLoading };
}

interface IOperatorSelectorProps {
    options: IOperatorOption[];
    selectedOption: IOperatorOption | null;
    isLoading: boolean;
    onSelect: (operatorId: string | null) => void;
    onSearchQueryChange: (query: string) => void;
}

/** The dialog's operator search. Ranking is done by `useOperatorOptions`, so the combobox's own filter is off. */
export function OperatorSelector({ options, selectedOption, isLoading, onSelect, onSearchQueryChange }: IOperatorSelectorProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("tools");

    return (
        <div className="space-y-2">
            <label className="block font-medium text-[13px] text-muted-foreground leading-none" htmlFor="operator-selector">
                {t("planner.dialog.selectOperator")}
            </label>
            <Combobox<IOperatorOption, false> items={options} value={selectedOption} onValueChange={(item) => onSelect(item?.id ?? null)} filter={null} onInputValueChange={onSearchQueryChange} itemToStringLabel={(op) => op?.displayName ?? ""} itemToStringValue={(op) => op?.id ?? ""}>
                <ComboboxInput id="operator-selector" placeholder={isLoading ? t("planner.dialog.loadingOperators") : t("planner.dialog.searchOperators")} />
                <ComboboxPopup className="max-w-100">
                    <ComboboxEmpty>{t("planner.dialog.noOperators")}</ComboboxEmpty>
                    <ComboboxList>
                        {(op: IOperatorOption) => (
                            <ComboboxItem key={op.id} value={op}>
                                <span className="flex w-full items-center gap-3">
                                    <span aria-hidden="true" className="relative flex size-9 shrink-0 items-center justify-center overflow-hidden rounded-lg border border-border bg-muted/50">
                                        <OperatorAvatar charId={op.id} name={op.displayName} className="block h-full w-full object-cover" server={op.isUpcoming ? "cn" : undefined} />
                                    </span>
                                    <span className="flex-1 font-medium text-foreground text-sm">{op.displayName}</span>
                                    <span className="font-normal text-muted-foreground text-xs">{t("planner.dialog.rarityClass", { rarity: op.rarity, class: professionLabel(op.profession) })}</span>
                                </span>
                            </ComboboxItem>
                        )}
                    </ComboboxList>
                </ComboboxPopup>
            </Combobox>
        </div>
    );
}

interface IOperatorMultiSelectorProps {
    options: IOperatorOption[];
    /** The picked operators, in the order they were picked. */
    selectedOptions: IOperatorOption[];
    isLoading: boolean;
    onSelectedChange: (options: IOperatorOption[]) => void;
    searchQuery: string;
    onSearchQueryChange: (query: string) => void;
}

/**
 * The bulk dialog's operator search: the same rows and ranking as
 * `OperatorSelector`, but every pick toggles in and the popup stays open.
 * The input is always the search box; the picks are listed by the dialog.
 */
export function OperatorMultiSelector({ options, selectedOptions, isLoading, onSelectedChange, searchQuery, onSearchQueryChange }: IOperatorMultiSelectorProps): React.ReactElement {
    const t: TypedT<typeof messages & typeof bulkMessages> = useT("tools");
    const selectedIds = React.useMemo(() => new Set(selectedOptions.map((op) => op.id)), [selectedOptions]);
    const unpickedMatches = options.filter((op) => !selectedIds.has(op.id));

    const addAllMatches = () => onSelectedChange([...selectedOptions, ...unpickedMatches]);

    return (
        <Combobox<IOperatorOption, true>
            multiple
            items={options}
            value={selectedOptions}
            onValueChange={onSelectedChange}
            filter={null}
            inputValue={searchQuery}
            onInputValueChange={onSearchQueryChange}
            isItemEqualToValue={(a, b) => a.id === b.id}
            itemToStringLabel={(op) => op?.displayName ?? ""}
            itemToStringValue={(op) => op?.id ?? ""}
        >
            <ComboboxInput id="bulk-operator-selector" placeholder={isLoading ? t("planner.dialog.loadingOperators") : t("planner.dialog.searchOperators")} />
            <ComboboxPopup className="max-w-100">
                {searchQuery.trim() && unpickedMatches.length > 0 && (
                    <div className="border-border border-b p-1">
                        <button type="button" onClick={addAllMatches} className="flex w-full items-center rounded-md px-2 py-1.5 text-left font-medium text-primary text-xs hover:bg-primary/10">
                            {t("planner.bulk.addAllResults", { count: unpickedMatches.length })}
                        </button>
                    </div>
                )}
                <ComboboxEmpty>{t("planner.dialog.noOperators")}</ComboboxEmpty>
                <ComboboxList>
                    {(op: IOperatorOption) => {
                        const isSelected = selectedIds.has(op.id);
                        return (
                            <ComboboxPrimitive.Item key={op.id} value={op} className="flex min-h-8 cursor-default items-center gap-3 rounded-sm px-2 py-1 text-sm outline-none data-disabled:pointer-events-none data-highlighted:bg-accent data-highlighted:text-accent-foreground data-disabled:opacity-64">
                                <span className={cn("flex size-4.5 shrink-0 items-center justify-center rounded-sm border transition-colors sm:size-4", isSelected ? "border-primary bg-primary text-primary-foreground" : "border-input bg-background")}>
                                    {isSelected && (
                                        <svg aria-hidden="true" className="size-3 sm:size-2.5" fill="none" height="24" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="3" viewBox="0 0 24 24" width="24" xmlns="http://www.w3.org/2000/svg">
                                            <path d="M5.252 12.7 10.2 18.63 18.748 5.37" />
                                        </svg>
                                    )}
                                </span>
                                <span aria-hidden="true" className="relative flex size-9 shrink-0 items-center justify-center overflow-hidden rounded-lg border border-border bg-muted/50">
                                    <OperatorAvatar charId={op.id} name={op.displayName} className="block h-full w-full object-cover" server={op.isUpcoming ? "cn" : undefined} />
                                </span>
                                <span className="flex-1 font-medium text-foreground text-sm">{op.displayName}</span>
                                <span className="font-normal text-muted-foreground text-xs">{t("planner.dialog.rarityClass", { rarity: op.rarity, class: professionLabel(op.profession) })}</span>
                            </ComboboxPrimitive.Item>
                        );
                    }}
                </ComboboxList>
            </ComboboxPopup>
        </Combobox>
    );
}
