import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { ChevronRight } from "lucide-react";
import * as React from "react";
import { Card, CardHeader, CardPanel, CardTitle } from "#/components/ui/card";
import { recruitmentDataQueryOptions } from "#/lib/api/recruitment";
import { Route } from "#/routes/tools.recruitment";
import { calculateResults } from "./impl/calculator";
import { CalculatorOptionsPanel } from "./impl/components/CalculatorOptionsPanel";
import { ResultsList } from "./impl/components/ResultsList";
import { SelectedTagsBar } from "./impl/components/SelectedTagsBar";
import { TagSelector } from "./impl/components/TagSelector";
import { MAX_SELECTED_TAGS } from "./impl/constants";
import { groupTagsByType, transformTags } from "./impl/helpers";
import type { ICalculatorOptions, IRecruitmentTag, OperatorSortMode } from "./impl/types";

const DEFAULT_OPTIONS: Required<ICalculatorOptions> = {
    includeRobots: true,
    operatorSortMode: "rarity-desc",
};

export function RecruitmentCalculator(): React.ReactElement {
    const { data } = useQuery(recruitmentDataQueryOptions());
    const { tags: selectedIds } = Route.useSearch();
    const navigate = useNavigate({ from: Route.fullPath });

    const tags = data?.tags ?? [];
    const operators = data?.operators ?? [];

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

    const setSelectedIds = React.useCallback(
        (updater: (prev: ReadonlyArray<number>) => ReadonlyArray<number>) => {
            navigate({
                search: (prev) => {
                    const next = updater(prev.tags ?? []);
                    return { ...prev, tags: [...next] };
                },
                replace: true,
                resetScroll: false,
            });
        },
        [navigate],
    );

    const onToggle = React.useCallback(
        (id: number) => {
            setSelectedIds((prev) => {
                if (prev.includes(id)) return prev.filter((x) => x !== id);
                if (prev.length >= MAX_SELECTED_TAGS) return prev;
                return [...prev, id];
            });
        },
        [setSelectedIds],
    );

    const onRemove = React.useCallback(
        (id: number) => {
            setSelectedIds((prev) => prev.filter((x) => x !== id));
        },
        [setSelectedIds],
    );

    const onReset = React.useCallback(() => {
        setSelectedIds(() => []);
    }, [setSelectedIds]);

    const onChangeIncludeRobots = React.useCallback((value: boolean) => {
        setOptions((prev) => ({ ...prev, includeRobots: value }));
    }, []);
    const onChangeSortMode = React.useCallback((value: OperatorSortMode) => {
        setOptions((prev) => ({ ...prev, operatorSortMode: value }));
    }, []);

    const selectedIdSet = React.useMemo(() => new Set(selectedIds), [selectedIds]);
    const maxReached = selectedIds.length >= MAX_SELECTED_TAGS;

    return (
        <div className="relative z-1 mx-auto w-[min(1400px,calc(100%-1.5rem))] py-4 pb-24 sm:w-[min(1400px,calc(100%-2rem))] sm:py-5 sm:pb-20">
            <nav aria-label="breadcrumb" className="mb-2.5 flex items-center gap-1.5 font-medium font-sans text-[12px] text-muted-foreground leading-none">
                <span>Tools</span>
                <ChevronRight className="size-2.5" />
                <span className="text-foreground">Recruitment Calculator</span>
            </nav>
            <div className="flex flex-wrap items-end justify-between gap-3">
                <div className="min-w-0 flex-1">
                    <h1 className="m-0 font-bold font-sans text-[24px] text-foreground leading-[1.1] tracking-tight sm:text-[30px]">Recruitment Calculator</h1>
                    <p className="mt-1.5 max-w-2xl font-sans text-[13px] text-muted-foreground leading-normal sm:text-[13.5px]">
                        Pick the tags shown in your recruitment screen - up to {MAX_SELECTED_TAGS}. Combinations that can give a 5★ rank first. Six-star operators only appear in combinations that include "Top Operator".
                    </p>
                </div>
            </div>

            <div className="mt-5 grid grid-cols-1 items-start gap-3 sm:mt-6 sm:gap-4 lg:grid-cols-[330px_1fr] xl:grid-cols-[400px_1fr]">
                <aside className="flex min-w-0 flex-col gap-3 sm:gap-4">
                    <Card>
                        <CardHeader className="p-4 sm:p-6">
                            <CardTitle className="text-[15px]">
                                Tags
                                <span className="ml-1.5 font-medium font-mono text-[11px] text-muted-foreground">
                                    ({selectedIds.length}/{MAX_SELECTED_TAGS})
                                </span>
                            </CardTitle>
                        </CardHeader>
                        <CardPanel className="px-4 pt-0 pb-4 sm:px-6 sm:pb-6">
                            <TagSelector groups={tagGroups} selectedTagIds={selectedIdSet} onToggle={onToggle} maxReached={maxReached} />
                        </CardPanel>
                    </Card>

                    <Card>
                        <CardHeader className="p-4 sm:p-6">
                            <CardTitle className="text-[15px]">Options</CardTitle>
                        </CardHeader>
                        <CardPanel className="px-4 pt-0 pb-4 sm:px-6 sm:pb-6">
                            <CalculatorOptionsPanel options={options} onChangeIncludeRobots={onChangeIncludeRobots} onChangeSortMode={onChangeSortMode} />
                        </CardPanel>
                    </Card>
                </aside>

                <main className="flex min-w-0 flex-col gap-3 sm:gap-4">
                    <SelectedTagsBar selectedTags={selectedTags} resultCount={results.length} onRemove={onRemove} onReset={onReset} />
                    <ResultsList results={results} hasSelection={selectedTags.length > 0} />
                </main>
            </div>
        </div>
    );
}
