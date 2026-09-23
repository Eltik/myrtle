import { useQuery } from "@tanstack/react-query";
import * as React from "react";

import { Combobox, ComboboxEmpty, ComboboxInput, ComboboxItem, ComboboxList, ComboboxPopup } from "#/components/ui/combobox";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { useOperatorName } from "#/hooks/use-operator-name";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { upcomingQueryOptions } from "#/lib/api/upcoming";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { professionLabel } from "#/lib/registry/operator-display";
import { searchAndRank } from "#/lib/search/fuzzy";
import type { IOperatorIndexEntry } from "#/types/operators";
import type { messages } from "./OperatorPlannerDialog.messages";

/** One row of the operator picker. */
interface IOperatorOption extends Pick<IOperatorIndexEntry, "id" | "name" | "appellation" | "rarity" | "profession" | "subProfessionId" | "tagList" | "nationId"> {
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

    return { options, selectedOption, isLoading: isOperatorsLoading || isUpcomingLoading };
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
