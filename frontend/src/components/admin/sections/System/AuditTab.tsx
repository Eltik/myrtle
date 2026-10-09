import { infiniteQueryOptions, useInfiniteQuery, useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import { AdminToolbar, skeletons } from "#/components/admin/Primitives";
import { TEXT_FIELDS } from "#/components/admin/sections/OperatorNotes/notesModel";
import { type ChangeKind, changeKind } from "#/components/admin/shell/model";
import { SearchField } from "#/components/SearchField";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card } from "#/components/ui/card";
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from "#/components/ui/empty";
import { useErrorMessage } from "#/components/ui/error-message";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "#/components/ui/table";
import { Tabs, TabsList, TabsTab } from "#/components/ui/tabs";
import { getGlobalAuditLogFn, getTranslationAuditLogFn, localesQueryOptions } from "#/lib/api/admin";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { useFormatters, useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { type AuditFieldFilter, type AuditSourceFilter, buildTimeline, filterTimeline, type IAuditRow, previewValue } from "./audit";
import type { messages } from "./System.messages";

type SystemT = TypedT<typeof messages>;

/** Rows per request to each log. The backend clamps notes at 500. */
const PAGE_SIZE = 50;

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

/**
 * Edit history: the operator notes log and the translations log merged into
 * one timeline. Each log pages on its own `before` cursor; `buildTimeline`
 * holds back rows a missing page could interleave with.
 */
export function AuditTab({ enabled }: { enabled: boolean }): React.ReactElement {
    const t: SystemT = useT("admin");
    const f = useFormatters();
    const describeError = useErrorMessage();
    const [source, setSource] = useState<AuditSourceFilter>("all");
    const [field, setField] = useState<AuditFieldFilter>("all");
    const [query, setQuery] = useState("");

    const notesQuery = useInfiniteQuery(cursorPagedOptions(["admin", "operator-notes", "audit", "global", "timeline", PAGE_SIZE], (before) => getGlobalAuditLogFn({ data: { limit: PAGE_SIZE, before } }), enabled));
    const translationsQuery = useInfiniteQuery(cursorPagedOptions(["admin", "i18n", "audit", "timeline", PAGE_SIZE], (before) => getTranslationAuditLogFn({ data: { limit: PAGE_SIZE, before } }), enabled));
    const operatorsQuery = useQuery({ ...operatorsIndexQueryOptions(useGamedataServer()), enabled });
    const localesQuery = useQuery({ ...localesQueryOptions(enabled), enabled });

    const opNames = useMemo(() => new Map((operatorsQuery.data ?? []).map((o) => [o.id, o.name])), [operatorsQuery.data]);
    const langNames = useMemo(() => new Map((localesQuery.data ?? []).map((l) => [l.code, l.english_name])), [localesQuery.data]);
    const opName = (id: string | null): string => (id ? (opNames.get(id) ?? id) : "");
    const langName = (code: string | null): string => (code ? (langNames.get(code) ?? code) : "");

    const notes = useMemo(() => notesQuery.data?.pages.flatMap((p) => p.entries) ?? [], [notesQuery.data]);
    const translations = useMemo(() => translationsQuery.data?.pages.flatMap((p) => p.entries) ?? [], [translationsQuery.data]);
    const notesTotal = notesQuery.data?.pages[0]?.total ?? 0;
    const translationsTotal = translationsQuery.data?.pages[0]?.total ?? 0;

    const timeline = useMemo(() => buildTimeline({ notes, translations, notesHasMore: notesQuery.hasNextPage, translationsHasMore: translationsQuery.hasNextPage, source }), [notes, translations, notesQuery.hasNextPage, translationsQuery.hasNextPage, source]);
    const rows = useMemo(() => filterTimeline(timeline.rows, field, query, (r) => (r.source === "notes" ? [opNames.get(r.operatorId ?? "") ?? ""] : [langNames.get(r.locale ?? "") ?? ""])), [timeline.rows, field, query, opNames, langNames]);

    const useNotes = source !== "translations";
    const useTranslations = source !== "notes";
    const loading = (useNotes && notesQuery.isPending) || (useTranslations && translationsQuery.isPending);
    const total = (useNotes ? notesTotal : 0) + (useTranslations ? translationsTotal : 0);
    const canLoadMore = (useNotes && notesQuery.hasNextPage) || (useTranslations && translationsQuery.hasNextPage);
    const loadingMore = notesQuery.isFetchingNextPage || translationsQuery.isFetchingNextPage;

    const loadMore = (): void => {
        if (useNotes && notesQuery.hasNextPage) void notesQuery.fetchNextPage();
        if (useTranslations && translationsQuery.hasNextPage) void translationsQuery.fetchNextPage();
    };

    const errors: { key: string; text: string; retry: () => void }[] = [];
    if (useNotes && notesQuery.isError) errors.push({ key: "notes", text: t("system.audit.error.notes", { message: describeError(notesQuery.error) }), retry: () => void notesQuery.refetch() });
    if (useTranslations && translationsQuery.isError) errors.push({ key: "translations", text: t("system.audit.error.translations", { message: describeError(translationsQuery.error) }), retry: () => void translationsQuery.refetch() });

    const fieldOptions: AuditFieldFilter[] = ["all", ...TEXT_FIELDS];
    const changeSource = (next: AuditSourceFilter): void => {
        setSource(next);
        // A note field means nothing on the translations log.
        if (next === "translations") setField("all");
    };

    return (
        <Card>
            <AdminToolbar>
                <Tabs value={source} onValueChange={(v) => changeSource(v as AuditSourceFilter)}>
                    <TabsList>
                        <TabsTab value="all">{t("system.audit.source.all")}</TabsTab>
                        <TabsTab value="notes">{t("system.audit.source.notes")}</TabsTab>
                        <TabsTab value="translations">{t("system.audit.source.translations")}</TabsTab>
                    </TabsList>
                </Tabs>
                <SearchField value={query} onChange={setQuery} placeholder={t("system.audit.search.placeholder")} label={t("system.audit.search.label")} className="w-full sm:w-60 md:w-70" />
                {source !== "translations" ? (
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
                            <AuditRow key={r.key} row={r} target={r.source === "notes" ? opName(r.operatorId) : t("system.audit.target.translation", { language: langName(r.locale), key: r.messageKey ?? "" })} />
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

/**
 * One edit. Below lg the change summary moves under the target (and below md
 * the time moves under the editor), so the row fits a tablet and a phone
 * without a sideways scroll. Each piece renders once and is placed twice.
 */
function AuditRow({ row, target }: { row: IAuditRow; target: string }): React.ReactElement {
    const t: SystemT = useT("admin");
    const f = useFormatters();
    const kind = changeKind(row.oldValue, row.newValue);
    const preview = previewValue(row.oldValue, row.newValue);
    const fullPreview = previewValue(row.oldValue, row.newValue, Number.POSITIVE_INFINITY);
    const when = (
        <span className="text-muted-foreground" title={f.date(row.changedAt, { dateStyle: "medium", timeStyle: "short" })}>
            {f.relativeShort(row.changedAt)}
        </span>
    );
    const change = (
        <>
            <Badge variant={KIND_VARIANT[kind]} size="sm">
                {kindLabel(t, kind)}
            </Badge>
            <span className="min-w-0 truncate text-muted-foreground leading-[1.45]" title={fullPreview}>
                {preview}
            </span>
        </>
    );
    return (
        <TableRow>
            <TableCell className="max-md:hidden">{when}</TableCell>
            <TableCell>
                <span className="flex flex-col gap-1">
                    {row.actor.uid ? (
                        <Link to="/user/$id" params={{ id: row.actor.uid }} title={row.actor.nickname ?? row.actor.uid} className="max-w-[24ch] truncate font-medium hover:underline max-md:max-w-[12ch]">
                            {row.actor.nickname ?? row.actor.uid}
                        </Link>
                    ) : (
                        <span title={t("system.audit.actor.deleted")} className="max-w-[24ch] truncate font-medium text-muted-foreground max-md:max-w-[12ch]">
                            {t("system.audit.actor.deleted")}
                        </span>
                    )}
                    <span className="text-[12.5px] md:hidden">{when}</span>
                </span>
            </TableCell>
            <TableCell>
                <span className="flex flex-col gap-1">
                    {row.source === "notes" && row.operatorId ? (
                        <Link to="/operators/$id" params={{ id: row.operatorId }} title={target} className="max-w-[24ch] truncate hover:underline max-md:max-w-[20ch]">
                            {target}
                        </Link>
                    ) : (
                        <span title={target} className="max-w-[24ch] truncate max-md:max-w-[20ch]">
                            {target}
                        </span>
                    )}
                    <span className="flex max-w-[24ch] items-center gap-2 text-[12.5px] md:max-w-[40ch] lg:hidden">{change}</span>
                </span>
            </TableCell>
            <TableCell className="max-md:hidden">
                <Badge variant="outline" size="sm">
                    {fieldLabel(t, row.field)}
                </Badge>
            </TableCell>
            <TableCell className="max-lg:hidden">
                <span className="inline-flex max-w-[380px] items-center gap-2 py-0.5 leading-[1.45]">{change}</span>
            </TableCell>
        </TableRow>
    );
}
