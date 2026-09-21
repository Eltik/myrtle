import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import * as React from "react";
import { Card, CardHeader, CardPanel, CardTitle } from "#/components/ui/card";
import { PageHeader } from "#/components/ui/page-header";
import { recruitmentDataQueryOptions } from "#/lib/api/recruitment";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { Route } from "#/routes/tools.recruitment";
import { calculateResults } from "./impl/calculator";
import { CalculatorOptionsPanel } from "./impl/components/CalculatorOptionsPanel";
import { ResultsList } from "./impl/components/ResultsList";
import { SelectedTagsBar } from "./impl/components/SelectedTagsBar";
import { TagSelector } from "./impl/components/TagSelector";
import { MAX_SELECTED_TAGS } from "./impl/constants";
import { groupTagsByType, transformTags } from "./impl/helpers";
import type { ICalculatorSettings, IRecruitmentTag, IRosterOverlay, IRosterViewOptions } from "./impl/types";
import { useRecruitRoster } from "./impl/useRecruitRoster";
import type { messages } from "./RecruitmentCalculator.messages";

const DEFAULT_SETTINGS: ICalculatorSettings = {
    includeRobots: true,
    includeTwoStars: true,
    includeThreeStars: true,
    operatorSortMode: "rarity-desc",
};

// Both overlays on by default: a signed-in reader opened the tool to see them.
const DEFAULT_ROSTER_VIEW: IRosterViewOptions = { showPotentials: true, showNextUpgrade: true };

export function RecruitmentCalculator(): React.ReactElement {
    const t: TypedT<typeof messages> = useT("tools");
    const { data } = useQuery(recruitmentDataQueryOptions(useGamedataServer()));
    const { tags: selectedIds } = Route.useSearch();
    const navigate = useNavigate({ from: Route.fullPath });

    const tags = data?.tags ?? [];
    const operators = data?.operators ?? [];

    const [settings, setSettings] = React.useState(DEFAULT_SETTINGS);
    const [rosterView, setRosterView] = React.useState(DEFAULT_ROSTER_VIEW);
    const roster = useRecruitRoster();

    // The potential sort is only offered while signed in; a session that ends
    // mid-visit falls back to the default rather than sorting on an empty map.
    const activeSettings: ICalculatorSettings = React.useMemo(() => {
        if (roster.signedIn || settings.operatorSortMode !== "potential-asc") return settings;
        return { ...settings, operatorSortMode: DEFAULT_SETTINGS.operatorSortMode };
    }, [settings, roster.signedIn]);

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

    const results = React.useMemo(() => {
        const potentialByOperator = roster.signedIn ? roster.potentialByOperator : undefined;
        return calculateResults(selectedTags, operators, { ...activeSettings, potentialByOperator });
    }, [selectedTags, operators, activeSettings, roster.signedIn, roster.potentialByOperator]);

    // Cards get the overlay only once the roster has landed: an empty map would
    // paint every operator as unowned for the length of the request.
    const rosterOverlay: IRosterOverlay | null = React.useMemo(() => {
        if (!roster.signedIn || roster.loading) return null;
        if (!rosterView.showPotentials && !rosterView.showNextUpgrade) return null;
        return { potentialByOperator: roster.potentialByOperator, ...rosterView };
    }, [roster.signedIn, roster.loading, roster.potentialByOperator, rosterView]);

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

    const onChangeSettings = React.useCallback((patch: Partial<ICalculatorSettings>) => {
        setSettings((prev) => ({ ...prev, ...patch }));
    }, []);
    const onChangeRosterView = React.useCallback((patch: Partial<IRosterViewOptions>) => {
        setRosterView((prev) => ({ ...prev, ...patch }));
    }, []);

    const selectedIdSet = React.useMemo(() => new Set(selectedIds), [selectedIds]);
    const maxReached = selectedIds.length >= MAX_SELECTED_TAGS;

    return (
        <div className="page-shell [--page-max:1400px]">
            <PageHeader breadcrumbLabel="breadcrumb" breadcrumb={[t("recruit.breadcrumb.tools"), t("recruit.title")]} title={t("recruit.title")} description={t("recruit.intro", { max: MAX_SELECTED_TAGS })} />

            <div className="mt-5 grid grid-cols-1 items-start gap-3 sm:mt-6 sm:gap-4 lg:grid-cols-[330px_1fr] xl:grid-cols-[400px_1fr]">
                <aside className="flex min-w-0 flex-col gap-3 sm:gap-4">
                    <Card>
                        <CardHeader className="p-4 sm:p-6">
                            <CardTitle className="text-[15px]">
                                {t("recruit.tags")}
                                <span className="ml-1.5 font-medium font-mono text-[11px] text-muted-foreground">{t("recruit.tags.count", { selected: selectedIds.length, max: MAX_SELECTED_TAGS })}</span>
                            </CardTitle>
                        </CardHeader>
                        <CardPanel className="px-4 pt-0 pb-4 sm:px-6 sm:pb-6">
                            <TagSelector groups={tagGroups} selectedTagIds={selectedIdSet} onToggle={onToggle} maxReached={maxReached} />
                        </CardPanel>
                    </Card>

                    <Card>
                        <CardHeader className="p-4 sm:p-6">
                            <CardTitle className="text-[15px]">{t("recruit.options")}</CardTitle>
                        </CardHeader>
                        <CardPanel className="px-4 pt-0 pb-4 sm:px-6 sm:pb-6">
                            <CalculatorOptionsPanel settings={activeSettings} rosterView={rosterView} rosterAvailable={roster.signedIn} onChangeSettings={onChangeSettings} onChangeRosterView={onChangeRosterView} />
                        </CardPanel>
                    </Card>
                </aside>

                <main className="flex min-w-0 flex-col gap-3 sm:gap-4">
                    <SelectedTagsBar selectedTags={selectedTags} resultCount={results.length} onRemove={onRemove} onReset={onReset} />
                    <ResultsList results={results} hasSelection={selectedTags.length > 0} roster={rosterOverlay} />
                </main>
            </div>
        </div>
    );
}
