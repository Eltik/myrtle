import { useQuery } from "@tanstack/react-query";
import { Link, useNavigate } from "@tanstack/react-router";
import { useEffect, useMemo, useState } from "react";
import { AdminToolbar, PanelMessage, stickyActionsCell } from "#/components/admin/Primitives";
import { levelVariant } from "#/components/admin/shell/model";
import type { AdminTierListsFilter, IAdminTierListsSearch } from "#/components/admin/shell/search";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card } from "#/components/ui/card";
import { InputGroup, InputGroupInput } from "#/components/ui/input-group";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Skeleton } from "#/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "#/components/ui/table";
import { Tabs, TabsList, TabsTab } from "#/components/ui/tabs";
import { useAuth } from "#/hooks/use-auth";
import { browseTierListsQueryOptions, grantedTierListsQueryOptions } from "#/lib/api/tier-lists";
import { useFormatters, useGamedataServer, useT } from "#/lib/i18n";
import { DeleteListDialog, FlairDialog } from "./dialogs";
import { COLUMN_COUNT, PublishRow, RowMenu } from "./ListRowActions";
import { type TierListsT, useLevelLabel } from "./labels";
import { canEditList, canPublishList, filterCounts, filterRows, grantedRows, type ITierListRow, LIST_TYPE_SCOPES, type ListTypeScope, staffRows } from "./model";

function scopeLabel(t: TierListsT, scope: ListTypeScope): string {
    switch (scope) {
        case "official":
            return t("tierLists.lists.type.official");
        case "community":
            return t("tierLists.lists.type.community");
        case "all":
            return t("tierLists.lists.type.all");
    }
}

function Dash(): React.ReactElement {
    return <span className="text-muted-foreground">—</span>;
}

export function ListsTab({ search, staff }: { search: IAdminTierListsSearch; staff: boolean }): React.ReactElement {
    const t: TierListsT = useT("admin");
    const f = useFormatters();
    const { isAuthenticated } = useAuth();
    const server = useGamedataServer();
    const navigate = useNavigate({ from: "/admin/tier-lists" });

    const browseQuery = useQuery(browseTierListsQueryOptions(server));
    // An editor's own grants; staff hold every list at admin and never ask.
    const grantedEnabled = isAuthenticated && !staff;
    const grantedQuery = useQuery({ ...grantedTierListsQueryOptions(grantedEnabled), enabled: grantedEnabled });

    const [scope, setScope] = useState<ListTypeScope>("official");
    const [q, setQ] = useState(search.q ?? "");
    const [publishing, setPublishing] = useState<string | null>(null);
    const [flairFor, setFlairFor] = useState<ITierListRow | null>(null);
    const [deleteFor, setDeleteFor] = useState<ITierListRow | null>(null);
    const filter: AdminTierListsFilter = search.filter ?? "all";

    // Back/forward or a deep link changes the URL under the input.
    useEffect(() => setQ(search.q ?? ""), [search.q]);

    const onQuery = (value: string): void => {
        setQ(value);
        void navigate({ search: (prev) => ({ ...prev, q: value || undefined }), replace: true });
    };
    const onFilter = (value: AdminTierListsFilter): void => {
        setPublishing(null);
        void navigate({ search: (prev) => ({ ...prev, filter: value === "all" ? undefined : value }), replace: true });
    };

    const rows = useMemo(() => {
        const browse = browseQuery.data ?? [];
        return staff ? staffRows(browse, scope) : grantedRows(browse, grantedQuery.data ?? []);
    }, [staff, browseQuery.data, grantedQuery.data, scope]);
    const shown = useMemo(() => filterRows(rows, filter, q), [rows, filter, q]);
    const counts = filterCounts(rows);
    const loading = browseQuery.isPending || (grantedEnabled && grantedQuery.isPending);
    const failed = browseQuery.isError || (grantedEnabled && grantedQuery.isError);

    let emptyText: string | null = null;
    if (!loading && !failed && shown.length === 0) {
        if (rows.length > 0) emptyText = t("tierLists.lists.empty.noMatch");
        else emptyText = staff ? t("tierLists.lists.empty.noLists") : t("tierLists.lists.empty.noAccess");
    }

    return (
        <Card>
            <AdminToolbar>
                <div className="w-full md:w-70">
                    <InputGroup>
                        <InputGroupInput size="sm" value={q} onChange={(e) => onQuery(e.target.value)} placeholder={t("tierLists.lists.searchPlaceholder")} aria-label={t("tierLists.lists.searchLabel")} />
                    </InputGroup>
                </div>
                <Tabs value={filter} onValueChange={(v) => onFilter(v as AdminTierListsFilter)}>
                    <TabsList>
                        <TabsTab value="all">{t("tierLists.lists.filter.all", { count: f.number(counts.all) })}</TabsTab>
                        <TabsTab value="hot">{t("tierLists.lists.filter.hot", { count: f.number(counts.hot) })}</TabsTab>
                    </TabsList>
                </Tabs>
                {staff ? (
                    <div className="md:ml-auto">
                        <Select
                            value={scope}
                            onValueChange={(next: string | null) => {
                                if (next === null) return;
                                setPublishing(null);
                                setScope(next as ListTypeScope);
                            }}
                        >
                            <SelectTrigger size="sm" className="w-auto" aria-label={t("tierLists.lists.type.label")}>
                                <SelectValue>{() => scopeLabel(t, scope)}</SelectValue>
                            </SelectTrigger>
                            <SelectContent>
                                {LIST_TYPE_SCOPES.map((s) => (
                                    <SelectItem key={s} value={s}>
                                        {scopeLabel(t, s)}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                ) : null}
            </AdminToolbar>

            {failed ? (
                <PanelMessage className="p-8">{t("tierLists.lists.loadError")}</PanelMessage>
            ) : (
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead className="pl-4.5">{t("tierLists.lists.th.title")}</TableHead>
                            <TableHead className="max-md:hidden">{t("tierLists.lists.th.flair")}</TableHead>
                            <TableHead className="max-lg:hidden">{t("tierLists.lists.th.trending")}</TableHead>
                            <TableHead className="max-md:hidden">{t("tierLists.lists.th.tiers")}</TableHead>
                            <TableHead className="max-md:hidden">{t("tierLists.lists.th.placements")}</TableHead>
                            <TableHead className="max-xl:hidden">{t("tierLists.lists.th.views")}</TableHead>
                            <TableHead>{t("tierLists.lists.th.updated")}</TableHead>
                            <TableHead className={stickyActionsCell}>
                                <span className="sr-only">{t("tierLists.lists.th.actions")}</span>
                            </TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {loading
                            ? [0, 1, 2, 3].map((i) => (
                                  <TableRow key={i}>
                                      <TableCell colSpan={COLUMN_COUNT}>
                                          <Skeleton className="h-9" />
                                      </TableCell>
                                  </TableRow>
                              ))
                            : shown.map((row) => (
                                  <ListRow key={row.slug} row={row} staff={staff} publishing={publishing === row.slug} onTogglePublish={() => setPublishing((cur) => (cur === row.slug ? null : row.slug))} onClosePublish={() => setPublishing(null)} onFlair={() => setFlairFor(row)} onDelete={() => setDeleteFor(row)} />
                              ))}
                    </TableBody>
                </Table>
            )}
            {emptyText ? <PanelMessage className="p-8">{emptyText}</PanelMessage> : null}

            <FlairDialog list={flairFor} onClose={() => setFlairFor(null)} />
            <DeleteListDialog
                list={deleteFor}
                onClose={() => setDeleteFor(null)}
                onDeleted={(slug) => {
                    if (publishing === slug) setPublishing(null);
                }}
            />
        </Card>
    );
}

interface IListRowProps {
    row: ITierListRow;
    staff: boolean;
    publishing: boolean;
    onTogglePublish: () => void;
    onClosePublish: () => void;
    onFlair: () => void;
    onDelete: () => void;
}

function ListRow({ row, staff, publishing, onTogglePublish, onClosePublish, onFlair, onDelete }: IListRowProps): React.ReactElement {
    const t: TierListsT = useT("admin");
    const levelLabel = useLevelLabel();
    const f = useFormatters();
    const color = row.flairColor ?? "var(--muted-foreground)";
    const editable = canEditList(row.level);

    return (
        <>
            <TableRow>
                <TableCell>
                    <div className="flex max-w-[32ch] flex-col gap-1 pl-2">
                        <span className="flex items-center gap-1.5 font-medium text-[14px] leading-tight">
                            <span className="min-w-0 truncate" title={row.title}>
                                {row.title}
                            </span>
                            {row.listType === "community" ? (
                                <Badge variant="outline" size="sm" className="shrink-0">
                                    {t("tierLists.lists.community")}
                                </Badge>
                            ) : null}
                        </span>
                        <span className="truncate text-[12px] text-muted-foreground leading-tight">
                            <span className="font-mono">/{row.slug}</span> · {row.author ?? t("tierLists.lists.unlisted")}
                        </span>
                    </div>
                </TableCell>
                <TableCell className="max-md:hidden">
                    {row.flairLabel ? (
                        <span className="inline-flex items-center gap-1.5 font-medium text-[13px]" style={{ color }}>
                            <span className="size-1.75 rounded-full" style={{ background: color }} />
                            {row.flairLabel}
                        </span>
                    ) : (
                        <Dash />
                    )}
                </TableCell>
                <TableCell className="max-lg:hidden">{row.hot ? <Badge variant="success">{t("tierLists.lists.trending")}</Badge> : <Dash />}</TableCell>
                <TableCell className="tabular-nums max-md:hidden">{row.tiers === null ? <Dash /> : f.number(row.tiers)}</TableCell>
                <TableCell className="tabular-nums max-md:hidden">{row.placements === null ? <Dash /> : f.number(row.placements)}</TableCell>
                <TableCell className="tabular-nums max-xl:hidden">{row.views24h === null ? <Dash /> : f.compact(row.views24h)}</TableCell>
                <TableCell className="text-muted-foreground">{row.updatedAtMs === null ? <Dash /> : f.relative(new Date(row.updatedAtMs).toISOString())}</TableCell>
                <TableCell className={stickyActionsCell}>
                    <div className="flex items-center justify-end gap-1.5 pr-1.5">
                        {staff ? null : <Badge variant={levelVariant(row.level, "default")}>{levelLabel(row.level)}</Badge>}
                        {editable ? (
                            <Button size="sm" variant="outline" render={<Link to="/tier-lists/my/$id/edit" params={{ id: row.slug }} />}>
                                {t("tierLists.lists.edit")}
                            </Button>
                        ) : (
                            <Button size="sm" variant="outline" render={<Link to="/tier-lists/$id" params={{ id: row.slug }} target="_blank" rel="noreferrer" />}>
                                {t("tierLists.lists.view")}
                            </Button>
                        )}
                        {canPublishList(row.level) ? (
                            <Button size="sm" variant={publishing ? "secondary" : "default"} onClick={onTogglePublish} aria-expanded={publishing}>
                                {t("tierLists.lists.publish")}
                            </Button>
                        ) : null}
                        <RowMenu row={row} onFlair={onFlair} onDelete={onDelete} />
                    </div>
                </TableCell>
            </TableRow>
            {publishing ? <PublishRow row={row} onClose={onClosePublish} /> : null}
        </>
    );
}
