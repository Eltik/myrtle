import { useQueries } from "@tanstack/react-query";
import { ChevronDown, ChevronRight, Info, LayoutList, List, Users } from "lucide-react";
import * as React from "react";

import { itemIcon } from "#/components/operators/detail/impl/assets";
import { Input } from "#/components/ui/input";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Skeleton } from "#/components/ui/skeleton";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { type IOperatorPlanResponse, type IPlanRequirementItem, plansQueryOptions } from "#/lib/api/planner";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { compactForSearch } from "#/lib/search/fuzzy";
import { cn } from "#/lib/utils";
import type { messages } from "./RequirementsPanel.messages";
import {
    type CategoryFilter,
    DEFAULT_REQUIREMENTS_VIEW,
    formatPlanTarget,
    formatSubtotal,
    type IRequirementsViewSettings,
    inCategory,
    inStatus,
    REQUIREMENT_CATEGORIES,
    REQUIREMENTS_VIEW_STORAGE_KEY,
    type RequirementCategory,
    type RequirementStatus,
    requirementCategory,
    requirementStatus,
    STATUS_FILTER_LABEL_KEYS,
    STATUS_FILTER_ORDER,
    type StatusFilter,
    subtotal,
} from "./requirements";
import type { messages as requirementMessages } from "./requirements.messages";

/** This panel renders its own chrome plus the labels `requirements.ts` carries. */
type ReqT = TypedT<typeof messages & typeof requirementMessages>;

interface PlannerRequirementRowProps {
    item: IPlanRequirementItem;
    depth: number;
    path: string;
    expandedPaths: Record<string, boolean>;
    onToggleExpand: (path: string) => void;
}

function PlannerRequirementRow({ item, depth, path, expandedPaths, onToggleExpand }: PlannerRequirementRowProps) {
    const t: ReqT = useT("tools");
    const f = useFormatters();
    const hasRecipe = !!(item.recipe && item.recipe.costs.length > 0);
    const isExpanded = expandedPaths[path];
    const isMissingRequirements = !item.canCraft && item.craftReason.startsWith("Requirements not met");
    const { status, shortfall } = requirementStatus(item);
    const needsCrafting = status === "craft";

    const handleClick = () => {
        if (hasRecipe) {
            onToggleExpand(path);
        }
    };

    return (
        <>
            <tr className={cn("group transition-colors hover:bg-muted/20", hasRecipe && "cursor-pointer")} onClick={handleClick}>
                <td className="py-2.5 pr-4">
                    <div className="flex items-center gap-2" style={{ paddingLeft: `${depth * 1.25}rem` }}>
                        {hasRecipe ? isExpanded ? <ChevronDown className="size-3.5 shrink-0 text-muted-foreground" /> : <ChevronRight className="size-3.5 shrink-0 text-muted-foreground" /> : <div className="size-3.5 shrink-0" />}
                        <img
                            src={itemIcon(item.id, item.iconId, item.image)}
                            alt={item.name}
                            className="size-8 shrink-0 rounded-md object-contain"
                            onError={(e) => {
                                (e.currentTarget as HTMLImageElement).style.display = "none";
                            }}
                        />
                        <span className="font-medium text-foreground text-xs leading-tight">{item.name}</span>
                    </div>
                </td>
                <td className="py-2.5 pr-2 text-right text-foreground text-xs tabular-nums">{f.number(item.requiredCount)}</td>
                <td className={cn("px-2 py-2.5 text-right text-xs tabular-nums", status === "complete" ? "text-emerald-400" : needsCrafting ? "text-amber-400" : "text-muted-foreground")}>{f.number(item.inventoryCount)}</td>
                <td className="px-2 py-2.5 text-right text-xs tabular-nums">
                    {item.canCraft ? (
                        <span className="text-sky-400">{f.number(item.craftableCount)}</span>
                    ) : isMissingRequirements ? (
                        <div className="flex justify-end">
                            <Tooltip>
                                <TooltipTrigger
                                    render={(props) => (
                                        <span {...props} className="inline-flex cursor-help items-center text-muted-foreground/60 hover:text-foreground">
                                            <Info className="size-3.5" />
                                        </span>
                                    )}
                                />
                                <TooltipPopup className="max-w-xs">{item.craftReason}</TooltipPopup>
                            </Tooltip>
                        </div>
                    ) : (
                        <span className="text-muted-foreground/50" title={item.craftReason}>
                            -
                        </span>
                    )}
                </td>
                <td className="py-2.5 pl-2 text-right text-xs tabular-nums">
                    {status === "missing" ? (
                        <span className="font-semibold text-red-400 text-xs">{f.number(item.missingCount)}</span>
                    ) : needsCrafting ? (
                        <span className="font-semibold text-amber-400 text-xs" title={t("planner.req.craftTitle", { have: f.number(item.inventoryCount), required: f.number(item.requiredCount), shortfall: f.number(shortfall) })}>
                            {t("planner.req.craft", { count: f.number(shortfall) })}
                        </span>
                    ) : (
                        <span className="font-semibold text-emerald-400">✓</span>
                    )}
                </td>
            </tr>
            {hasRecipe &&
                isExpanded &&
                item.recipe?.costs.map((cost) => {
                    const childPath = `${path}/${cost.item.id}`;
                    return <PlannerRequirementRow key={childPath} item={cost.item} depth={depth + 1} path={childPath} expandedPaths={expandedPaths} onToggleExpand={onToggleExpand} />;
                })}
        </>
    );
}

/** Dropdown option row: label left, count right, matching the table's tabular numerals. */
function FilterOption({ label, count }: { label: string; count: number }) {
    return (
        <span className="flex w-full items-center justify-between gap-3">
            <span>{label}</span>
            <span className="text-muted-foreground text-xs tabular-nums">{count}</span>
        </span>
    );
}

function TableHead({ t }: { t: ReqT }) {
    return (
        <thead>
            <tr className="border-border/40 border-b">
                <th className="pb-2 text-left font-medium text-muted-foreground text-xs">{t("planner.req.col.item")}</th>
                <th className="pr-2 pb-2 text-right font-medium text-muted-foreground text-xs">{t("planner.req.col.required")}</th>
                <th className="px-2 pb-2 text-right font-medium text-muted-foreground text-xs">{t("planner.req.col.have")}</th>
                <th className="px-2 pb-2 text-right font-medium text-muted-foreground text-xs">{t("planner.req.col.craftable")}</th>
                <th className="pb-2 pl-2 text-right font-medium text-muted-foreground text-xs">{t("planner.req.col.missing")}</th>
            </tr>
        </thead>
    );
}

interface RequirementsTableProps {
    items: IPlanRequirementItem[];
    grouped: boolean;
    pathPrefix?: string;
    expandedPaths: Record<string, boolean>;
    onToggleExpand: (path: string) => void;
    /** Only read when `grouped`. */
    collapsedGroups?: Record<string, boolean>;
    onToggleGroup?: (key: string) => void;
}

function RequirementsTable({ items, grouped, pathPrefix = "", expandedPaths, onToggleExpand, collapsedGroups, onToggleGroup }: RequirementsTableProps) {
    const t: ReqT = useT("tools");
    const rowFor = (item: IPlanRequirementItem) => <PlannerRequirementRow key={`${pathPrefix}${item.id}`} item={item} depth={0} path={`${pathPrefix}${item.id}`} expandedPaths={expandedPaths} onToggleExpand={onToggleExpand} />;

    if (!grouped) {
        return (
            <table className="w-full text-sm">
                <TableHead t={t} />
                <tbody className="divide-y divide-border/30">{items.map(rowFor)}</tbody>
            </table>
        );
    }

    const groups = REQUIREMENT_CATEGORIES.map((def) => ({ def, rows: items.filter((it) => requirementCategory(it) === def.key) })).filter((g) => g.rows.length > 0);

    return (
        <table className="w-full text-sm">
            <TableHead t={t} />
            {groups.map((g) => {
                const collapsed = collapsedGroups?.[g.def.key] ?? false;
                return (
                    <tbody key={g.def.key} className="divide-y divide-border/30">
                        <tr className="bg-muted/20">
                            <td colSpan={5} className="p-0">
                                <button type="button" onClick={() => onToggleGroup?.(g.def.key)} className="flex w-full items-center gap-2 px-1 py-2 text-left transition-colors hover:bg-muted/30">
                                    {collapsed ? <ChevronRight className="size-3.5 shrink-0 text-muted-foreground" /> : <ChevronDown className="size-3.5 shrink-0 text-muted-foreground" />}
                                    <span className="font-semibold text-foreground text-xs">{t(g.def.labelKey)}</span>
                                    <span className="text-muted-foreground text-xs">{formatSubtotal(subtotal(g.rows), t)}</span>
                                </button>
                            </td>
                        </tr>
                        {!collapsed && g.rows.map(rowFor)}
                    </tbody>
                );
            })}
        </table>
    );
}

interface ByOperatorSectionProps {
    plan: IOperatorPlanResponse;
    predicate: (item: IPlanRequirementItem) => boolean;
    isLoading: boolean;
    requirements: IPlanRequirementItem[];
    collapsed: boolean;
    onToggle: () => void;
    expandedPaths: Record<string, boolean>;
    onToggleExpand: (path: string) => void;
}

function ByOperatorSection({ plan, predicate, isLoading, requirements, collapsed, onToggle, expandedPaths, onToggleExpand }: ByOperatorSectionProps) {
    const t: ReqT = useT("tools");
    const op = plan.operator;
    const filtered = requirements.filter(predicate);

    return (
        <div className="overflow-hidden rounded-lg border border-border/60 bg-muted/5">
            <button type="button" onClick={onToggle} className="flex w-full items-center gap-3 p-3 text-left transition-colors hover:bg-muted/20">
                {collapsed ? <ChevronRight className="size-4 shrink-0 text-muted-foreground" /> : <ChevronDown className="size-4 shrink-0 text-muted-foreground" />}
                <span aria-hidden="true" className="relative flex size-8 shrink-0 items-center justify-center overflow-hidden rounded-lg border border-border bg-muted/70">
                    <OperatorAvatar charId={op.id} name={op.name} className="block h-full w-full object-cover" server={op.server} />
                </span>
                <div className="min-w-0 flex-1">
                    <span className="block truncate font-semibold text-foreground text-xs leading-tight">{op.name}</span>
                    <span className="block truncate text-[11px] text-muted-foreground leading-normal">{formatPlanTarget(plan, t)}</span>
                </div>
                {!isLoading && <span className="shrink-0 text-[11px] text-muted-foreground">{formatSubtotal(subtotal(filtered), t)}</span>}
            </button>
            {!collapsed && (
                <div className="border-border/40 border-t px-3 py-3">
                    {isLoading ? (
                        <div className="flex flex-col gap-3">
                            {["a", "b", "c"].map((k) => (
                                <div key={k} className="flex items-center gap-3">
                                    <Skeleton className="size-8 shrink-0 rounded-md" />
                                    <Skeleton className="h-3 w-24 rounded" />
                                    <Skeleton className="ml-auto h-3 w-16 rounded" />
                                </div>
                            ))}
                        </div>
                    ) : filtered.length === 0 ? (
                        <p className="py-3 text-center text-muted-foreground text-xs">{t("planner.req.noneMatchingOperator")}</p>
                    ) : (
                        <div className="overflow-x-auto">
                            <RequirementsTable items={filtered} grouped={false} pathPrefix={`${plan.operator_id}::`} expandedPaths={expandedPaths} onToggleExpand={onToggleExpand} />
                        </div>
                    )}
                </div>
            )}
        </div>
    );
}

interface ByOperatorViewProps {
    plans: IOperatorPlanResponse[];
    predicate: (item: IPlanRequirementItem) => boolean;
    collapsedSections: Record<string, boolean>;
    onToggleSection: (id: string) => void;
    expandedPaths: Record<string, boolean>;
    onToggleExpand: (path: string) => void;
}

function ByOperatorView({ plans, predicate, collapsedSections, onToggleSection, expandedPaths, onToggleExpand }: ByOperatorViewProps) {
    const t: ReqT = useT("tools");
    const results = useQueries({
        queries: plans.map((p) => plansQueryOptions([p.operator_id])),
    });

    if (plans.length === 0) {
        return <p className="mt-6 text-center text-muted-foreground text-sm">{t("planner.req.noActivePlans")}</p>;
    }

    return (
        <div className="mt-3 flex flex-col gap-3">
            {plans.map((p, i) => {
                const q = results[i];
                return (
                    <ByOperatorSection
                        key={p.operator_id}
                        plan={p}
                        predicate={predicate}
                        isLoading={q.isLoading}
                        requirements={q.data?.aggregatedRequirements ?? []}
                        collapsed={collapsedSections[p.operator_id] ?? false}
                        onToggle={() => onToggleSection(p.operator_id)}
                        expandedPaths={expandedPaths}
                        onToggleExpand={onToggleExpand}
                    />
                );
            })}
        </div>
    );
}

interface RequirementsPanelProps {
    aggregatedRequirements: IPlanRequirementItem[];
    isLoading: boolean;
    activePlans: IOperatorPlanResponse[];
}

export function RequirementsPanel({ aggregatedRequirements, isLoading, activePlans }: RequirementsPanelProps): React.ReactElement {
    const t: ReqT = useT("tools");
    const [settings, setSettings] = useLocalStorageState<IRequirementsViewSettings>(REQUIREMENTS_VIEW_STORAGE_KEY, DEFAULT_REQUIREMENTS_VIEW);
    const [reqSearchQuery, setReqSearchQuery] = React.useState("");
    const [expandedPaths, setExpandedPaths] = React.useState<Record<string, boolean>>({});
    const [collapsedGroups, setCollapsedGroups] = React.useState<Record<string, boolean>>({});
    const [collapsedSections, setCollapsedSections] = React.useState<Record<string, boolean>>({});

    const handleToggleExpand = (path: string) => setExpandedPaths((prev) => ({ ...prev, [path]: !prev[path] }));
    const handleToggleGroup = (key: string) => setCollapsedGroups((prev) => ({ ...prev, [key]: !prev[key] }));
    const handleToggleSection = (id: string) => setCollapsedSections((prev) => ({ ...prev, [id]: !prev[id] }));

    const searchQuery = compactForSearch(reqSearchQuery);
    const matchesSearch = React.useCallback((item: IPlanRequirementItem) => compactForSearch(item.name).includes(searchQuery), [searchQuery]);

    // Chip counts derive from the shared aggregate so they stay stable across views.
    const searchFiltered = React.useMemo(() => aggregatedRequirements.filter(matchesSearch), [aggregatedRequirements, matchesSearch]);

    const categoryCounts = React.useMemo(() => {
        const counts: Record<RequirementCategory, number> = { expLmd: 0, skills: 0, chips: 0, modules: 0, materials: 0 };
        for (const item of searchFiltered) counts[requirementCategory(item)]++;
        return counts;
    }, [searchFiltered]);

    const categoryFiltered = React.useMemo(() => searchFiltered.filter((r) => inCategory(r, settings.category)), [searchFiltered, settings.category]);

    const statusCounts = React.useMemo(() => {
        const counts: Record<RequirementStatus, number> = { missing: 0, craft: 0, complete: 0 };
        for (const item of categoryFiltered) counts[requirementStatus(item).status]++;
        return counts;
    }, [categoryFiltered]);

    const predicate = React.useCallback((item: IPlanRequirementItem) => matchesSearch(item) && inCategory(item, settings.category) && inStatus(item, settings.status), [matchesSearch, settings.category, settings.status]);

    const aggregateFiltered = React.useMemo(() => aggregatedRequirements.filter(predicate), [aggregatedRequirements, predicate]);

    const setCategory = (category: CategoryFilter) => setSettings((prev) => ({ ...prev, category }));
    const setStatus = (status: StatusFilter) => setSettings((prev) => ({ ...prev, status }));

    const hasRequirements = !isLoading && aggregatedRequirements.length > 0;

    return (
        <div className="rounded-xl border border-border bg-card p-4">
            <div className="flex flex-wrap items-center justify-between gap-3">
                <h2 className="font-semibold text-foreground text-sm">{t("planner.req.title")}</h2>
                {hasRequirements && (
                    <div className="flex flex-wrap items-center gap-2">
                        <Input type="search" placeholder={t("planner.req.search")} value={reqSearchQuery} onChange={(e) => setReqSearchQuery(e.target.value)} className="h-7 max-w-48 text-xs sm:h-7 sm:text-xs" />
                        <ToggleGroup
                            aria-label={t("planner.req.view")}
                            variant="outline"
                            value={[settings.view]}
                            onValueChange={(value) => {
                                const next = value[0] as IRequirementsViewSettings["view"] | undefined;
                                if (next) setSettings((prev) => ({ ...prev, view: next }));
                            }}
                        >
                            <ToggleGroupItem value="grouped" aria-label={t("planner.req.view.grouped.aria")} title={t("planner.req.view.grouped")}>
                                <LayoutList />
                            </ToggleGroupItem>
                            <ToggleGroupItem value="flat" aria-label={t("planner.req.view.flat.aria")} title={t("planner.req.view.flat")}>
                                <List />
                            </ToggleGroupItem>
                            <ToggleGroupItem value="by-operator" aria-label={t("planner.req.view.byOperator.aria")} title={t("planner.req.view.byOperator")}>
                                <Users />
                            </ToggleGroupItem>
                        </ToggleGroup>
                    </div>
                )}
            </div>

            {hasRequirements && (
                <div className="mt-3 flex flex-wrap items-center gap-2">
                    <Select value={settings.category} onValueChange={(v) => v != null && setCategory(v as CategoryFilter)}>
                        <SelectTrigger size="sm" className={cn("h-7 text-xs", settings.category !== "all" && "border-primary/50")}>
                            <SelectValue placeholder={t("planner.req.filter.type")}>
                                {() => {
                                    const def = REQUIREMENT_CATEGORIES.find((d) => d.key === settings.category);
                                    const value = settings.category === "all" || !def ? t("planner.req.filter.all") : t(def.labelKey);
                                    return t("planner.req.filter.typeValue", { value });
                                }}
                            </SelectValue>
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="all">
                                <FilterOption label={t("planner.req.filter.all")} count={searchFiltered.length} />
                            </SelectItem>
                            {REQUIREMENT_CATEGORIES.map((def) => (
                                <SelectItem key={def.key} value={def.key} disabled={categoryCounts[def.key] === 0}>
                                    <FilterOption label={t(def.labelKey)} count={categoryCounts[def.key]} />
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                    <Select value={settings.status} onValueChange={(v) => v != null && setStatus(v as StatusFilter)}>
                        <SelectTrigger size="sm" className={cn("h-7 text-xs", settings.status !== "all" && "border-primary/50")}>
                            <SelectValue placeholder={t("planner.req.filter.status")}>{() => t("planner.req.filter.statusValue", { value: settings.status === "all" ? t("planner.req.filter.all") : t(STATUS_FILTER_LABEL_KEYS[settings.status]) })}</SelectValue>
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="all">
                                <FilterOption label={t("planner.req.filter.all")} count={categoryFiltered.length} />
                            </SelectItem>
                            {STATUS_FILTER_ORDER.map((s) => (
                                <SelectItem key={s} value={s} disabled={statusCounts[s] === 0}>
                                    <FilterOption label={t(STATUS_FILTER_LABEL_KEYS[s])} count={statusCounts[s]} />
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                </div>
            )}

            {isLoading ? (
                <div className="mt-4 flex flex-col divide-y divide-border/40">
                    {["sk-req-1", "sk-req-2", "sk-req-3", "sk-req-4", "sk-req-5", "sk-req-6", "sk-req-7", "sk-req-8"].map((key) => (
                        <div key={key} className="flex items-center gap-3 py-3 first:pt-1">
                            <Skeleton className="size-9 shrink-0 rounded-lg" />
                            <Skeleton className="h-3.5 w-24 rounded" />
                            <div className="ml-auto flex gap-6">
                                <Skeleton className="h-3 w-8 rounded" />
                                <Skeleton className="h-3 w-8 rounded" />
                                <Skeleton className="h-3 w-8 rounded" />
                                <Skeleton className="h-3 w-8 rounded" />
                            </div>
                        </div>
                    ))}
                </div>
            ) : aggregatedRequirements.length === 0 ? (
                <p className="mt-6 text-center text-muted-foreground text-sm">{t("planner.req.noneForPlans")}</p>
            ) : settings.view === "by-operator" ? (
                <ByOperatorView plans={activePlans} predicate={predicate} collapsedSections={collapsedSections} onToggleSection={handleToggleSection} expandedPaths={expandedPaths} onToggleExpand={handleToggleExpand} />
            ) : aggregateFiltered.length === 0 ? (
                <p className="py-6 text-center text-muted-foreground text-sm">{t("planner.req.noneMatching")}</p>
            ) : (
                <div className="mt-3 overflow-x-auto">
                    <RequirementsTable items={aggregateFiltered} grouped={settings.view === "grouped"} expandedPaths={expandedPaths} onToggleExpand={handleToggleExpand} collapsedGroups={collapsedGroups} onToggleGroup={handleToggleGroup} />
                </div>
            )}
        </div>
    );
}
