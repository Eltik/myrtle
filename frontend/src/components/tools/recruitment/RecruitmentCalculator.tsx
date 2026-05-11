import { useQuery } from "@tanstack/react-query";
import { ChevronRight } from "lucide-react";
import * as React from "react";
import { Card, CardHeader, CardPanel, CardTitle } from "#/components/ui/card";
import { recruitmentDataQueryOptions } from "#/lib/api/recruitment";
import { calculateResults } from "./impl/calculator";
import { CalculatorOptionsPanel } from "./impl/components/CalculatorOptionsPanel";
import { ResultsList } from "./impl/components/ResultsList";
import { SelectedTagsBar } from "./impl/components/SelectedTagsBar";
import { TagSelector } from "./impl/components/TagSelector";
import { MAX_SELECTED_TAGS } from "./impl/constants";
import { groupTagsByType, transformTags } from "./impl/helpers";
import type { ICalculatorOptions, IRecruitmentTag, OperatorSortMode } from "./impl/types";

const DEFAULT_OPTIONS: Required<ICalculatorOptions> = {
    showLowRarity: false,
    includeRobots: true,
    operatorSortMode: "rarity-desc",
};

export function RecruitmentCalculator(): React.ReactElement {
    const { data } = useQuery(recruitmentDataQueryOptions());

    const tags = data?.tags ?? [];
    const operators = data?.operators ?? [];

    const [selectedIds, setSelectedIds] = React.useState<ReadonlyArray<number>>([]);
    const [options, setOptions] = React.useState<Required<ICalculatorOptions>>(DEFAULT_OPTIONS);

    const allTags: IRecruitmentTag[] = React.useMemo(() => transformTags(tags), [tags]);
    const tagGroups = React.useMemo(() => groupTagsByType(allTags), [allTags]);
    const tagById = React.useMemo(() => new Map(allTags.map((t) => [t.id, t])), [allTags]);

    const selectedTags: IRecruitmentTag[] = React.useMemo(() => {
        const out: IRecruitmentTag[] = [];
        for (const id of selectedIds) {
            const tag = tagById.get(id);
            if (tag) out.push(tag);
        }
        return out;
    }, [selectedIds, tagById]);

    const results = React.useMemo(() => calculateResults(selectedTags, operators, options), [selectedTags, operators, options]);

    const onToggle = React.useCallback((id: number) => {
        setSelectedIds((prev) => {
            if (prev.includes(id)) return prev.filter((x) => x !== id);
            if (prev.length >= MAX_SELECTED_TAGS) return prev;
            return [...prev, id];
        });
    }, []);

    const onRemove = React.useCallback((id: number) => {
        setSelectedIds((prev) => prev.filter((x) => x !== id));
    }, []);

    const onReset = React.useCallback(() => {
        setSelectedIds([]);
    }, []);

    const onChangeShowLowRarity = React.useCallback((value: boolean) => {
        setOptions((prev) => ({ ...prev, showLowRarity: value }));
    }, []);
    const onChangeIncludeRobots = React.useCallback((value: boolean) => {
        setOptions((prev) => ({ ...prev, includeRobots: value }));
    }, []);
    const onChangeSortMode = React.useCallback((value: OperatorSortMode) => {
        setOptions((prev) => ({ ...prev, operatorSortMode: value }));
    }, []);

    const selectedIdSet = React.useMemo(() => new Set(selectedIds), [selectedIds]);
    const maxReached = selectedIds.length >= MAX_SELECTED_TAGS;

    return (
        <div className="relative z-1 mx-auto w-[min(1400px,calc(100%-2rem))] py-5 pb-20">
            <nav aria-label="breadcrumb" className="mb-2.5 flex items-center gap-1.5 font-medium font-sans text-[12px] leading-none text-muted-foreground">
                <span>Tools</span>
                <ChevronRight className="size-2.5" />
                <span className="text-foreground">Recruitment Calculator</span>
            </nav>
            <div className="flex flex-wrap items-end justify-between gap-3">
                <div className="min-w-0 flex-1">
                    <h1 className="m-0 font-bold font-sans text-[24px] leading-[1.1] tracking-tight text-foreground sm:text-[30px]">Recruitment Calculator</h1>
                    <p className="mt-1.5 max-w-2xl font-sans text-[13.5px] leading-normal text-muted-foreground">Pick the tags shown in your recruitment screen — up to {MAX_SELECTED_TAGS}. Combinations are ranked by guaranteed minimum rarity. Six-star operators only appear when "Top Operator" is selected.</p>
                </div>
            </div>

            <div className="mt-6 grid grid-cols-1 items-start gap-4 xl:grid-cols-[420px_1fr]">
                <aside className="flex min-w-0 flex-col gap-4">
                    <Card>
                        <CardHeader>
                            <CardTitle className="text-[15px]">
                                Tags
                                <span className="ml-1.5 font-mono text-[11px] font-medium text-muted-foreground">
                                    ({selectedIds.length}/{MAX_SELECTED_TAGS})
                                </span>
                            </CardTitle>
                        </CardHeader>
                        <CardPanel className="pt-0">
                            <TagSelector groups={tagGroups} selectedTagIds={selectedIdSet} onToggle={onToggle} maxReached={maxReached} />
                        </CardPanel>
                    </Card>

                    <Card>
                        <CardHeader>
                            <CardTitle className="text-[15px]">Options</CardTitle>
                        </CardHeader>
                        <CardPanel className="pt-0">
                            <CalculatorOptionsPanel options={options} onChangeShowLowRarity={onChangeShowLowRarity} onChangeIncludeRobots={onChangeIncludeRobots} onChangeSortMode={onChangeSortMode} />
                        </CardPanel>
                    </Card>
                </aside>

                <main className="flex min-w-0 flex-col gap-4">
                    <SelectedTagsBar selectedTags={selectedTags} onRemove={onRemove} onReset={onReset} />
                    <ResultsList results={results} hasSelection={selectedTags.length > 0} />
                </main>
            </div>
        </div>
    );
}
