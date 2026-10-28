import { infiniteQueryOptions, useInfiniteQuery, useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { ListFilterIcon, XIcon } from "lucide-react";
import { useMemo, useState } from "react";
import { AdminToolbar, skeletons } from "#/components/admin/Primitives";
import { TEXT_FIELDS } from "#/components/admin/sections/OperatorNotes/notesModel";
import { useLevelLabel } from "#/components/admin/sections/TierLists/labels";
import { type ChangeKind, changeKind } from "#/components/admin/shell/model";
import type { AdminAuditSource, AdminAuditTargetKind, IAdminSystemSearch } from "#/components/admin/shell/search";
import { SearchField } from "#/components/SearchField";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card } from "#/components/ui/card";
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from "#/components/ui/empty";
import { useErrorMessage } from "#/components/ui/error-message";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "#/components/ui/table";
import { Tabs, TabsList, TabsTab } from "#/components/ui/tabs";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { type IPersonOption, PersonCombobox } from "#/components/user/PersonCombobox";
import { useDebounce } from "#/hooks/use-debounce";
import { adminUsersQueryOptions, getContentAuditLogFn, getGlobalAuditLogFn, getTranslationAuditLogFn, type IContentAuditEntry, localesQueryOptions, type TierListPermissionLevel } from "#/lib/api/admin";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { useFormatters, useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { type AuditFieldFilter, type AuditSourceFilter, buildTimeline, filterTimeline, type IAuditRow, previewValue } from "./audit";
import { actedOnOthers, type ContentChange, contentChanges } from "./contentAudit";
import type { messages } from "./System.messages";

type SystemT = TypedT<typeof messages>;

/** Rows per request to each log. The backend clamps notes at 500. */
const PAGE_SIZE = 50;

/** Tier names a summary clause lists before "…". */
const NAMES_SHOWN = 3;

const KIND_VARIANT: Record<ChangeKind, "success" | "destructive" | "info"> = { added: "success", cleared: "destructive", changed: "info" };

function fieldLabel(t: SystemT, field: string): string {
    switch (field) {
        case "summary":
            return t("system.audit.field.summary");
        case "pros":
            return t("system.audit.field.pros");
        case "cons":
            return t("system.audit.field.cons");
        case "notes":
            return t("system.audit.field.notes");
        case "trivia":
            return t("system.audit.field.trivia");
        case "text":
            return t("system.audit.field.text");
        default:
            // A column added to the audit after this list: show it rather than hide it.
            return field.length > 0 ? field[0].toUpperCase() + field.slice(1) : field;
    }
}

function kindLabel(t: SystemT, kind: ChangeKind): string {
    switch (kind) {
        case "added":
            return t("system.audit.kind.added");
        case "cleared":
            return t("system.audit.kind.cleared");
        case "changed":
            return t("system.audit.kind.changed");
    }
}

function actionLabel(t: SystemT, action: IContentAuditEntry["action"]): string {
    switch (action) {
        case "create":
            return t("system.audit.action.create");
        case "fork":
            return t("system.audit.action.fork");
        case "edit":
            return t("system.audit.action.edit");
        case "publish":
            return t("system.audit.action.publish");
        case "visibility":
            return t("system.audit.action.visibility");
        case "flair":
            return t("system.audit.action.flair");
        case "grant":
            return t("system.audit.action.grant");
        case "revoke":
            return t("system.audit.action.revoke");
        case "delete":
            return t("system.audit.action.delete");
    }
}

function actionVariant(action: IContentAuditEntry["action"]): "success" | "destructive" | "info" | "warning" {
    switch (action) {
        case "create":
        case "fork":
        case "grant":
            return "success";
        case "delete":
        case "revoke":
            return "destructive";
        case "visibility":
        case "flair":
            return "warning";
        default:
            return "info";
    }
}

function targetKindLabel(t: SystemT, kind: AdminAuditTargetKind | "all"): string {
    switch (kind) {
        case "all":
            return t("system.audit.targetKind.all");
        case "tier_list":
            return t("system.audit.targetKind.tier_list");
        case "grid":
            return t("system.audit.targetKind.grid");
    }
}

function names(list: readonly string[]): string {
    return list.length > NAMES_SHOWN ? `${list.slice(0, NAMES_SHOWN).join(", ")}…` : list.join(", ");
}

/** Words one content row's summary: its clauses through the catalog, joined. */
function useContentSummary(): (entry: IContentAuditEntry) => string {
    const t: SystemT = useT("admin");
    const levelLabel = useLevelLabel();
    const clause = (entry: IContentAuditEntry, c: ContentChange): string => {
        switch (c.kind) {
            case "listType":
                return c.listType === "official" ? t("system.audit.change.official") : t("system.audit.change.community");
            case "forkedFrom":
                return t("system.audit.change.forkedFrom", { title: c.title });
            case "renamed":
                return t("system.audit.change.renamed", { from: c.from });
            case "description":
                return t("system.audit.change.description");
            case "kinds":
                return t("system.audit.change.kinds", { count: c.count });
            case "listed":
                return c.listed ? t("system.audit.change.shown") : t("system.audit.change.hidden");
            case "size":
                return t("system.audit.change.size", { size: c.size });
            case "cells":
                return t("system.audit.change.cells", { count: c.count });
            case "tiersAdded":
                return t("system.audit.change.tiersAdded", { count: c.names.length, names: names(c.names) });
            case "tierRenamed":
                return t("system.audit.change.tierRenamed", { from: c.from, to: c.to });
            case "more":
                return t("system.audit.change.moreRenames", { count: c.count });
            case "tiersReordered":
                return t("system.audit.change.tiersReordered", { count: c.count });
            case "tiersRestyled":
                return t("system.audit.change.tiersRestyled", { names: names(c.names) });
            case "tiersDeleted":
                return c.dropped > 0 ? t("system.audit.change.tiersDeletedWith", { count: c.names.length, names: names(c.names), dropped: c.dropped }) : t("system.audit.change.tiersDeleted", { count: c.names.length, names: names(c.names) });
            case "entriesAdded":
                return t("system.audit.change.entriesAdded", { count: c.count });
            case "entriesMoved":
                return t("system.audit.change.entriesMoved", { count: c.count });
            case "entriesReordered":
                return t("system.audit.change.entriesReordered", { count: c.count });
            case "entriesRemoved":
                return t("system.audit.change.entriesRemoved", { count: c.count });
            case "notesEdited":
                return t("system.audit.change.notesEdited", { count: c.count });
            case "published":
                return c.changelog ? t("system.audit.change.publishedWith", { version: c.version, changelog: c.changelog }) : t("system.audit.change.published", { version: c.version });
            case "flair":
                return c.label ? t("system.audit.change.flair", { label: c.label }) : t("system.audit.change.flairCleared");
            case "permission": {
                const who = entry.subject ? (entry.subject.nickname ?? entry.subject.uid ?? entry.subject.user_id) : t("system.audit.change.someone");
                return t("system.audit.change.permission", { level: levelLabel(c.level as TierListPermissionLevel), name: who });
            }
            case "deleted":
                return t("system.audit.change.deleted", { tiers: c.tiers ?? 0, entries: c.entries ?? 0 });
        }
    };
    return (entry) =>
        contentChanges(entry)
            .map((c) => clause(entry, c))
            .join(" · ");
}

/** One log, paged newest first on a `before` cursor (the oldest loaded row's time). */
function cursorPagedOptions<TPage extends { entries: readonly { changed_at: string }[] }>(queryKey: readonly unknown[], fetchPage: (before: string | undefined) => Promise<TPage>, enabled: boolean) {
    return infiniteQueryOptions({
        queryKey,
        queryFn: ({ pageParam }) => fetchPage(pageParam),
        initialPageParam: undefined as string | undefined,
        getNextPageParam: (last) => (last.entries.length < PAGE_SIZE ? undefined : last.entries[last.entries.length - 1]?.changed_at),
        enabled,
        staleTime: 30 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

/** The edit history's filters, as the System route's search params hold them. */
export type IAuditFilters = Pick<IAdminSystemSearch, "source" | "actor" | "kind" | "target">;

interface IAuditTabProps {
    enabled: boolean;
    filters: IAuditFilters;
    onFiltersChange: (next: IAuditFilters) => void;
}

/**
 * Edit history: the operator notes log, the translations log and the tier
 * list and grid log merged into one timeline. Each log pages on its own
 * `before` cursor; `buildTimeline` holds back rows a missing page could
 * interleave with. The person, type and one-list filters run on the server
 * and live in the URL, so a filtered history is a link.
 */
export function AuditTab({ enabled, filters, onFiltersChange }: IAuditTabProps): React.ReactElement {
    const t: SystemT = useT("admin");
    const f = useFormatters();
    const describeError = useErrorMessage();
    const [field, setField] = useState<AuditFieldFilter>("all");
    const [query, setQuery] = useState("");
    const [actorQuery, setActorQuery] = useState("");
    const [pickedActor, setPickedActor] = useState<IPersonOption | null>(null);

    // A type or one-list filter only means something on the tier list and grid log.
    const source: AuditSourceFilter = filters.target || filters.kind ? "content" : (filters.source ?? "all");
    const actor = filters.actor;
    const useNotes = source === "all" || source === "notes";
    const useTranslations = source === "all" || source === "translations";
    const useContent = source === "all" || source === "content";

    const notesQuery = useInfiniteQuery(cursorPagedOptions(["admin", "operator-notes", "audit", "global", "timeline", PAGE_SIZE, actor ?? null], (before) => getGlobalAuditLogFn({ data: { limit: PAGE_SIZE, before, actor } }), enabled && useNotes));
    const translationsQuery = useInfiniteQuery(cursorPagedOptions(["admin", "i18n", "audit", "timeline", PAGE_SIZE, actor ?? null], (before) => getTranslationAuditLogFn({ data: { limit: PAGE_SIZE, before, actor } }), enabled && useTranslations));
    const contentQuery = useInfiniteQuery(
        cursorPagedOptions(["admin", "content-audit", "timeline", PAGE_SIZE, actor ?? null, filters.kind ?? null, filters.target ?? null], (before) => getContentAuditLogFn({ data: { limit: PAGE_SIZE, before, actor, targetKind: filters.kind, targetId: filters.target } }), enabled && useContent),
    );
    const operatorsQuery = useQuery({ ...operatorsIndexQueryOptions(useGamedataServer()), enabled });
    const localesQuery = useQuery({ ...localesQueryOptions(enabled), enabled });

    const actorTerm = useDebounce(actorQuery.trim(), 300);
    const actorSearchEnabled = enabled && actorTerm.length > 0;
    const actorSearch = useQuery({ ...adminUsersQueryOptions({ q: actorTerm, limit: 8 }, actorSearchEnabled), enabled: actorSearchEnabled });

    const opNames = useMemo(() => new Map((operatorsQuery.data ?? []).map((o) => [o.id, o.name])), [operatorsQuery.data]);
    const langNames = useMemo(() => new Map((localesQuery.data ?? []).map((l) => [l.code, l.english_name])), [localesQuery.data]);
    const opName = (id: string | null): string => (id ? (opNames.get(id) ?? id) : "");
    const langName = (code: string | null): string => (code ? (langNames.get(code) ?? code) : "");
    const summarize = useContentSummary();

    const notes = useMemo(() => notesQuery.data?.pages.flatMap((p) => p.entries) ?? [], [notesQuery.data]);
    const translations = useMemo(() => translationsQuery.data?.pages.flatMap((p) => p.entries) ?? [], [translationsQuery.data]);
    const content = useMemo(() => contentQuery.data?.pages.flatMap((p) => p.entries) ?? [], [contentQuery.data]);
    const notesTotal = notesQuery.data?.pages[0]?.total ?? 0;
    const translationsTotal = translationsQuery.data?.pages[0]?.total ?? 0;
    const contentTotal = contentQuery.data?.pages[0]?.total ?? 0;

    const timeline = useMemo(
        () => buildTimeline({ notes, translations, content, notesHasMore: notesQuery.hasNextPage, translationsHasMore: translationsQuery.hasNextPage, contentHasMore: contentQuery.hasNextPage, source }),
        [notes, translations, content, notesQuery.hasNextPage, translationsQuery.hasNextPage, contentQuery.hasNextPage, source],
    );
    const rows = useMemo(
        () =>
            filterTimeline(timeline.rows, field, query, (r) => {
                if (r.source === "notes") return [opNames.get(r.operatorId ?? "") ?? ""];
                if (r.source === "translations") return [langNames.get(r.locale ?? "") ?? ""];
                return r.content ? [summarize(r.content)] : [];
            }),
        [timeline.rows, field, query, opNames, langNames, summarize],
    );

    // The picked person's name: from the box, else from a loaded row (a shared link), else the bare id.
    const actorOption = useMemo<IPersonOption | null>(() => {
        if (!actor) return null;
        if (pickedActor?.id === actor) return pickedActor;
        const row = [...notes.map((n) => n.actor), ...translations.map((x) => x.actor), ...content.map((c) => c.actor)].find((a) => a.user_id === actor);
        return { id: actor, name: row?.nickname ?? row?.uid ?? actor.slice(0, 8), hint: row?.uid ?? null };
    }, [actor, pickedActor, notes, translations, content]);
    const actorOptions = useMemo<IPersonOption[]>(() => (actorTerm.length > 0 ? (actorSearch.data?.users ?? []).map((u) => ({ id: u.id, name: u.nickname ?? u.uid, hint: u.uid })) : []), [actorTerm, actorSearch.data]);
    const targetName = filters.target ? (content.find((c) => c.target_id === filters.target)?.target_name ?? t("system.audit.target.unknown")) : "";

    const loading = (useNotes && notesQuery.isPending) || (useTranslations && translationsQuery.isPending) || (useContent && contentQuery.isPending);
    const total = (useNotes ? notesTotal : 0) + (useTranslations ? translationsTotal : 0) + (useContent ? contentTotal : 0);
    const canLoadMore = (useNotes && notesQuery.hasNextPage) || (useTranslations && translationsQuery.hasNextPage) || (useContent && contentQuery.hasNextPage);
    const loadingMore = notesQuery.isFetchingNextPage || translationsQuery.isFetchingNextPage || contentQuery.isFetchingNextPage;

    const loadMore = (): void => {
        if (useNotes && notesQuery.hasNextPage) void notesQuery.fetchNextPage();
        if (useTranslations && translationsQuery.hasNextPage) void translationsQuery.fetchNextPage();
        if (useContent && contentQuery.hasNextPage) void contentQuery.fetchNextPage();
    };

    const errors: { key: string; text: string; retry: () => void }[] = [];
    if (useNotes && notesQuery.isError) errors.push({ key: "notes", text: t("system.audit.error.notes", { message: describeError(notesQuery.error) }), retry: () => void notesQuery.refetch() });
    if (useTranslations && translationsQuery.isError) errors.push({ key: "translations", text: t("system.audit.error.translations", { message: describeError(translationsQuery.error) }), retry: () => void translationsQuery.refetch() });
    if (useContent && contentQuery.isError) errors.push({ key: "content", text: t("system.audit.error.content", { message: describeError(contentQuery.error) }), retry: () => void contentQuery.refetch() });

    const fieldOptions: AuditFieldFilter[] = ["all", ...TEXT_FIELDS];
    const kindOptions: (AdminAuditTargetKind | "all")[] = ["all", "tier_list", "grid"];
    const changeSource = (next: AuditSourceFilter): void => {
        // Leaving the tier list and grid log drops its own filters.
        onFiltersChange({ actor, source: next === "all" ? undefined : (next as AdminAuditSource), kind: next === "content" ? filters.kind : undefined, target: next === "content" ? filters.target : undefined });
        // A note field means nothing on the other logs.
        if (next !== "notes" && next !== "all") setField("all");
    };
    const setActor = (next: IPersonOption | null): void => {
        setPickedActor(next);
        onFiltersChange({ ...filters, actor: next?.id });
    };
    const filterToTarget = (entry: IContentAuditEntry): void => onFiltersChange({ actor, source: "content", target: entry.target_id });

    return (
        <Card>
            <AdminToolbar>
                <Tabs value={source} onValueChange={(v) => changeSource(v as AuditSourceFilter)}>
                    <TabsList>
                        <TabsTab value="all">{t("system.audit.source.all")}</TabsTab>
                        <TabsTab value="notes">{t("system.audit.source.notes")}</TabsTab>
                        <TabsTab value="translations">{t("system.audit.source.translations")}</TabsTab>
                        <TabsTab value="content">{t("system.audit.source.content")}</TabsTab>
                    </TabsList>
                </Tabs>
                <SearchField value={query} onChange={setQuery} placeholder={t("system.audit.search.placeholder")} label={t("system.audit.search.label")} className="w-full sm:w-60 md:w-70" />
                <div className="w-full sm:w-52">
                    <PersonCombobox
                        value={actorOption}
                        onChange={setActor}
                        options={actorOptions}
                        onQueryChange={setActorQuery}
                        loading={actorSearchEnabled && (actorSearch.isPending || actorTerm !== actorQuery.trim())}
                        label={t("system.audit.actor.label")}
                        placeholder={t("system.audit.actor.placeholder")}
                        emptyText={t("system.audit.actor.empty")}
                    />
                </div>
                {source === "all" || source === "notes" ? (
                    <Select value={field} onValueChange={(v) => v && setField(v as AuditFieldFilter)}>
                        <SelectTrigger size="sm" className="w-full min-w-0 sm:w-40" aria-label={t("system.audit.field.label")}>
                            <SelectValue>{(value) => (value === "all" ? t("system.audit.field.all") : fieldLabel(t, String(value)))}</SelectValue>
                        </SelectTrigger>
                        <SelectContent>
                            {fieldOptions.map((opt) => (
                                <SelectItem key={opt} value={opt}>
                                    {opt === "all" ? t("system.audit.field.all") : fieldLabel(t, opt)}
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                ) : null}
                {source === "content" && !filters.target ? (
                    <Select value={filters.kind ?? "all"} onValueChange={(v) => v && onFiltersChange({ ...filters, source: "content", kind: v === "all" ? undefined : (v as AdminAuditTargetKind) })}>
                        <SelectTrigger size="sm" className="w-full min-w-0 sm:w-40" aria-label={t("system.audit.targetKind.label")}>
                            <SelectValue>{(value) => targetKindLabel(t, value as AdminAuditTargetKind | "all")}</SelectValue>
                        </SelectTrigger>
                        <SelectContent>
                            {kindOptions.map((opt) => (
                                <SelectItem key={opt} value={opt}>
                                    {targetKindLabel(t, opt)}
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                ) : null}
                {filters.target ? (
                    <Badge variant="outline" size="lg" className="max-w-full gap-1 pe-0.5">
                        <span className="truncate">{t("system.audit.target.chip", { name: targetName })}</span>
                        <Button variant="ghost" size="icon-xs" aria-label={t("system.audit.target.clear")} title={t("system.audit.target.clear")} onClick={() => onFiltersChange({ actor, source: "content" })}>
                            <XIcon />
                        </Button>
                    </Badge>
                ) : null}
                <span className="text-[12.5px] text-muted-foreground md:ml-auto">{t("system.audit.untracked")}</span>
            </AdminToolbar>

            {errors.map((e) => (
                <div key={e.key} className="flex flex-wrap items-center gap-3 border-border border-b bg-destructive/8 px-3.5 py-2.5 text-[12.5px] text-destructive-foreground">
                    <span className="min-w-0 flex-1">{e.text}</span>
                    <Button variant="outline" size="xs" onClick={e.retry}>
                        {t("system.audit.retry")}
                    </Button>
                </div>
            ))}

            {loading ? (
                <div className="flex flex-col gap-2 p-4">{skeletons(6, "h-9")}</div>
            ) : rows.length === 0 ? (
                errors.length > 0 && timeline.rows.length === 0 ? null : (
                    <Empty className="py-14">
                        <EmptyHeader>
                            <EmptyTitle>{timeline.rows.length === 0 ? t("system.audit.empty.title") : t("system.audit.noMatch.title")}</EmptyTitle>
                            <EmptyDescription>{timeline.rows.length === 0 ? t("system.audit.empty.desc") : t("system.audit.noMatch.desc")}</EmptyDescription>
                        </EmptyHeader>
                    </Empty>
                )
            ) : (
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead className="max-md:hidden">{t("system.audit.th.when")}</TableHead>
                            <TableHead>{t("system.audit.th.who")}</TableHead>
                            <TableHead>{t("system.audit.th.what")}</TableHead>
                            <TableHead className="max-md:hidden">{t("system.audit.th.field")}</TableHead>
                            <TableHead className="max-lg:hidden">{t("system.audit.th.change")}</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {rows.map((r) => (
                            <AuditRow
                                key={r.key}
                                row={r}
                                target={r.source === "notes" ? opName(r.operatorId) : r.source === "translations" ? t("system.audit.target.translation", { language: langName(r.locale), key: r.messageKey ?? "" }) : (r.content?.target_name ?? "")}
                                summary={r.content ? summarize(r.content) : null}
                                filteredActor={actor === r.actor.userId}
                                onFilterActor={() => setActor({ id: r.actor.userId, name: r.actor.nickname ?? r.actor.uid ?? r.actor.userId, hint: r.actor.uid })}
                                onFilterTarget={r.content && filters.target !== r.content.target_id ? filterToTarget : null}
                            />
                        ))}
                    </TableBody>
                </Table>
            )}

            {!loading && (rows.length > 0 || canLoadMore) ? (
                <div className="flex flex-wrap items-center justify-between gap-3 border-border border-t px-3.5 py-3">
                    <span className="text-[12.5px] text-muted-foreground tabular-nums">{t("system.audit.count", { count: total, shown: f.number(rows.length), total: f.number(total) })}</span>
                    {canLoadMore ? (
                        <Button variant="outline" size="sm" loading={loadingMore} disabled={loadingMore} onClick={loadMore}>
                            {t("system.audit.loadMore")}
                        </Button>
                    ) : null}
                </div>
            ) : null}
        </Card>
    );
}

interface IAuditRowProps {
    row: IAuditRow;
    target: string;
    /** Content rows: the worded summary. */
    summary: string | null;
    /** The person filter already shows only this row's actor. */
    filteredActor: boolean;
    onFilterActor: () => void;
    onFilterTarget: ((entry: IContentAuditEntry) => void) | null;
}

/** A small icon button that narrows the history to what sits beside it. */
function FilterButton({ label, onClick }: { label: string; onClick: () => void }): React.ReactElement {
    return (
        <Button variant="ghost" size="icon-xs" aria-label={label} title={label} onClick={onClick} className="shrink-0 text-muted-foreground opacity-0 pointer-coarse:opacity-100 transition-opacity focus-visible:opacity-100 group-hover/row:opacity-100">
            <ListFilterIcon />
        </Button>
    );
}

/** Where a tier list or grid lives, or nothing once it is deleted (its slug no longer resolves). */
function ContentTargetLink({ entry, label }: { entry: IContentAuditEntry; label: string }): React.ReactElement {
    const className = "max-w-[24ch] truncate hover:underline max-md:max-w-[20ch]";
    if (entry.action === "delete" || !entry.target_slug) {
        return (
            <span title={label} className={`max-w-[24ch] truncate max-md:max-w-[20ch] ${entry.action === "delete" ? "text-muted-foreground line-through" : ""}`}>
                {label}
            </span>
        );
    }
    return entry.target_kind === "grid" ? (
        <Link to="/grids/$slug" params={{ slug: entry.target_slug }} title={label} className={className}>
            {label}
        </Link>
    ) : (
        <Link to="/tier-lists/$id" params={{ id: entry.target_slug }} title={label} className={className}>
            {label}
        </Link>
    );
}

/**
 * One edit. Below lg the change summary moves under the target (and below md
 * the time moves under the editor), so the row fits a tablet and a phone
 * without a sideways scroll. Each piece renders once and is placed twice.
 */
function AuditRow({ row, target, summary, filteredActor, onFilterActor, onFilterTarget }: IAuditRowProps): React.ReactElement {
    const t: SystemT = useT("admin");
    const f = useFormatters();
    const entry = row.content;
    const when = (
        <Tooltip>
            <TooltipTrigger render={<span className="text-muted-foreground" />}>{f.relativeShort(row.changedAt)}</TooltipTrigger>
            <TooltipPopup>{entry && entry.event_count > 1 ? `${f.date(entry.first_at, { dateStyle: "medium", timeStyle: "short" })} – ${f.date(row.changedAt, { timeStyle: "short" })}` : f.date(row.changedAt, { dateStyle: "medium", timeStyle: "short" })}</TooltipPopup>
        </Tooltip>
    );

    let change: React.ReactElement;
    if (entry) {
        change = (
            <>
                <span className="min-w-0 truncate text-muted-foreground leading-[1.45]" title={summary ?? ""}>
                    {summary}
                </span>
                {entry.event_count > 1 ? <span className="shrink-0 font-mono text-[11px] text-muted-foreground/80 tabular-nums">{t("system.audit.events", { count: entry.event_count })}</span> : null}
            </>
        );
    } else {
        const kind = changeKind(row.oldValue, row.newValue);
        change = (
            <>
                <Badge variant={KIND_VARIANT[kind]} size="sm">
                    {kindLabel(t, kind)}
                </Badge>
                <span className="min-w-0 truncate text-muted-foreground leading-[1.45]" title={previewValue(row.oldValue, row.newValue, Number.POSITIVE_INFINITY)}>
                    {previewValue(row.oldValue, row.newValue)}
                </span>
            </>
        );
    }

    const owner = entry && actedOnOthers(entry) ? entry.owner : null;
    const actorName = row.actor.nickname ?? row.actor.uid ?? "";
    return (
        <TableRow className="group/row">
            <TableCell className="max-md:hidden">{when}</TableCell>
            <TableCell>
                <span className="flex flex-col gap-1">
                    <span className="flex min-w-0 items-center gap-1">
                        {row.actor.uid ? (
                            <Link to="/user/$id" params={{ id: row.actor.uid }} title={actorName} className="max-w-[24ch] truncate font-medium hover:underline max-md:max-w-[12ch]">
                                {actorName}
                            </Link>
                        ) : (
                            <span title={t("system.audit.actor.deleted")} className="max-w-[24ch] truncate font-medium text-muted-foreground max-md:max-w-[12ch]">
                                {t("system.audit.actor.deleted")}
                            </span>
                        )}
                        {filteredActor ? null : <FilterButton label={t("system.audit.actor.only", { name: actorName })} onClick={onFilterActor} />}
                    </span>
                    <span className="text-[12.5px] md:hidden">{when}</span>
                </span>
            </TableCell>
            <TableCell>
                <span className="flex flex-col gap-1">
                    <span className="flex min-w-0 items-center gap-1">
                        {entry ? (
                            <ContentTargetLink entry={entry} label={target} />
                        ) : row.source === "notes" && row.operatorId ? (
                            <Link to="/operators/$id" params={{ id: row.operatorId }} title={target} className="max-w-[24ch] truncate hover:underline max-md:max-w-[20ch]">
                                {target}
                            </Link>
                        ) : (
                            <span title={target} className="max-w-[24ch] truncate max-md:max-w-[20ch]">
                                {target}
                            </span>
                        )}
                        {entry && onFilterTarget ? <FilterButton label={t("system.audit.target.only")} onClick={() => onFilterTarget(entry)} /> : null}
                    </span>
                    {owner ? (
                        <span className="text-[12px] text-muted-foreground">
                            {owner.uid ? (
                                <Link to="/user/$id" params={{ id: owner.uid }} className="hover:underline">
                                    {t("system.audit.target.owner", { name: owner.nickname ?? owner.uid })}
                                </Link>
                            ) : (
                                t("system.audit.target.ownerGone")
                            )}
                        </span>
                    ) : null}
                    {entry ? (
                        <span className="md:hidden">
                            <Badge variant={actionVariant(entry.action)} size="sm">
                                {actionLabel(t, entry.action)}
                            </Badge>
                        </span>
                    ) : null}
                    <span className="flex max-w-[24ch] items-center gap-2 text-[12.5px] md:max-w-[40ch] lg:hidden">{change}</span>
                </span>
            </TableCell>
            <TableCell className="max-md:hidden">
                {entry ? (
                    <Badge variant={actionVariant(entry.action)} size="sm">
                        {actionLabel(t, entry.action)}
                    </Badge>
                ) : (
                    <Badge variant="outline" size="sm">
                        {fieldLabel(t, row.field)}
                    </Badge>
                )}
            </TableCell>
            <TableCell className="max-lg:hidden">
                <span className="inline-flex max-w-[380px] items-center gap-2 py-0.5 leading-[1.45]">{change}</span>
            </TableCell>
        </TableRow>
    );
}
