import { useQueries, useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useEffect, useRef, useState } from "react";
import { AdminToolbar, PanelMessage } from "#/components/admin/Primitives";
import { useAdminAccess } from "#/components/admin/shell/access";
import { useAdminLocales } from "#/components/admin/shell/locales";
import { useRoleLabel } from "#/components/admin/shell/nav";
import { PageHead } from "#/components/admin/shell/PageHead";
import type { AdminUsersFilter, IAdminUsersSearch } from "#/components/admin/shell/search";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card } from "#/components/ui/card";
import { useErrorMessage } from "#/components/ui/error-message";
import { InputGroup, InputGroupInput } from "#/components/ui/input-group";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "#/components/ui/table";
import { Tabs, TabsList, TabsTab } from "#/components/ui/tabs";
import { useAuth } from "#/hooks/use-auth";
import { adminUsersQueryOptions, type IAdminUser } from "#/lib/api/admin";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { roleVariant } from "./labels";
import { PersonAvatar } from "./PersonAvatar";
import { PersonSheet } from "./PersonSheet";
import type { messages } from "./Users.messages";
import { usePeopleGrants, usePeopleRoster } from "./usePeopleData";
import { type AccessSummary, accessSummary, accessWarning, grantsOf, holderOf, isDeletedAccount, PEOPLE_SERVERS, parsePeopleServer, personLabel } from "./warnings";

type UsersT = TypedT<typeof messages>;

export interface IUsersProps {
    search: IAdminUsersSearch;
}

const PAGE_SIZE = 50;
const SEARCH_DEBOUNCE_MS = 250;
/** Filter tabs on a phone: desktop's text size and a tighter inset, so the role row fits 375px. The strips still scroll for longer translations. */
const PHONE_TAB = "max-sm:px-2 max-sm:text-sm";

function accessText(t: UsersT, summary: AccessSummary): string {
    // Language grants count on any role, so they follow whatever the role gives.
    const withLanguages = (base: string, locales: readonly string[]) => (locales.length ? t("users.access.withLanguages", { access: base, languages: locales.join(", ") }) : base);
    switch (summary.kind) {
        case "everything":
            return t("users.access.everything");
        case "allLists":
            return withLanguages(t("users.access.allLists"), summary.locales);
        case "editor":
            return withLanguages(summary.lists.length ? t("users.access.editor", { lists: summary.lists.join(", ") }) : t("users.access.editorNoLists"), summary.locales);
        case "translator":
            return summary.locales.length ? summary.locales.join(", ") : t("users.access.translatorNone");
        case "player":
            return summary.locales.length ? summary.locales.join(", ") : "—";
    }
}

/** People (spec 2.2): every account, filterable by role and server, with a detail sheet per person. */
export default function Users({ search }: IUsersProps): React.ReactElement {
    const t: UsersT = useT("admin");
    const f = useFormatters();
    const navigate = useNavigate();
    const describeError = useErrorMessage();
    const roleLabel = useRoleLabel();
    const { user: me, isAuthenticated } = useAuth();
    const access = useAdminAccess();
    const staff = isAuthenticated && access.staff;

    const filter: AdminUsersFilter = search.filter ?? "all";
    const server = parsePeopleServer(search.server);
    const q = search.q?.trim() ?? "";

    const setSearch = (patch: Partial<IAdminUsersSearch>, replace = false) => {
        void navigate({ to: "/admin/users", search: { ...search, ...patch }, replace });
    };

    // The box types into local state; the URL (and the query) follow after a pause.
    const [draft, setDraft] = useState(search.q ?? "");
    const sentQ = useRef(search.q ?? "");
    const latestSearch = useRef(search);
    latestSearch.current = search;
    useEffect(() => {
        const urlQ = search.q ?? "";
        if (urlQ !== sentQ.current) {
            sentQ.current = urlQ;
            setDraft(urlQ);
        }
    }, [search.q]);
    useEffect(() => {
        const next = draft.trim();
        if (next === (sentQ.current ?? "").trim()) return;
        const id = setTimeout(() => {
            sentQ.current = next;
            void navigate({ to: "/admin/users", search: { ...latestSearch.current, q: next || undefined }, replace: true });
        }, SEARCH_DEBOUNCE_MS);
        return () => clearTimeout(id);
    }, [draft, navigate]);

    // "Show more" pages; a filter change starts over at one page.
    const filterKey = `${q}|${filter}|${server ?? ""}`;
    const [paging, setPaging] = useState({ key: filterKey, pages: 1 });
    const pages = paging.key === filterKey ? paging.pages : 1;
    const pageQueries = useQueries({
        queries: Array.from({ length: pages }, (_, i) => ({ ...adminUsersQueryOptions({ q: q || undefined, role: filter, server, limit: PAGE_SIZE, offset: i * PAGE_SIZE }, staff), enabled: staff })),
    });
    const rows = pageQueries.flatMap((pq) => pq.data?.users ?? []);
    const total = pageQueries[0]?.data?.total;
    const firstPage = pageQueries[0];
    const loadingMore = pageQueries.some((pq) => pq.isFetching);

    const grants = usePeopleGrants(staff);
    const roster = usePeopleRoster(staff);
    const { list: locales, name: localeName } = useAdminLocales(staff);

    // The open person: from the table, else from the staff roster, else by UID
    // when a grant names one (a Player linked from the inbox may be off-page).
    const selectedId = search.user;
    const listed = selectedId ? (rows.find((u) => u.id === selectedId) ?? roster?.byId.get(selectedId)) : undefined;
    const fallbackUid = selectedId && !listed && grants ? (holderOf(grantsOf(grants.byUser, selectedId)).uid ?? undefined) : undefined;
    const lookup = useQuery({ ...adminUsersQueryOptions({ q: fallbackUid, limit: 5 }, staff), enabled: staff && fallbackUid !== undefined });
    const lookedUp = lookup.data?.users.find((u) => u.id === selectedId);
    const stillResolving = firstPage?.isPending || !roster || !grants || (fallbackUid !== undefined && lookup.isPending);
    // An id with grants but no account row is a deleted account, not a bad link.
    const gone = selectedId && grants ? isDeletedAccount(grantsOf(grants.byUser, selectedId)) : false;
    const person: IAdminUser | "missing" | "deleted" | null = !selectedId ? null : (listed ?? lookedUp ?? (stillResolving ? null : gone ? "deleted" : "missing"));

    const filterTabs: { id: AdminUsersFilter; label: string }[] = [
        { id: "all", label: t("users.filter.all") },
        { id: "staff", label: t("users.filter.staff") },
        { id: "translators", label: t("users.filter.translators") },
        { id: "players", label: t("users.filter.players") },
    ];

    return (
        <>
            <PageHead kicker={t("users.head.kicker")} title={t("users.head.title")} sub={t("users.head.sub")} />
            <Card className="overflow-hidden">
                <AdminToolbar>
                    <div className="w-full md:w-70">
                        <InputGroup>
                            <InputGroupInput size="sm" type="search" value={draft} onChange={(e) => setDraft(e.target.value)} placeholder={t("users.search.placeholder")} aria-label={t("users.search.label")} />
                        </InputGroup>
                    </div>
                    <div className="max-w-full overflow-x-auto [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
                        <Tabs value={filter} onValueChange={(v) => setSearch({ filter: v === "all" ? undefined : (v as AdminUsersFilter) })}>
                            <TabsList aria-label={t("users.filter.label")}>
                                {filterTabs.map((tab) => (
                                    <TabsTab key={tab.id} value={tab.id} className={PHONE_TAB}>
                                        {tab.label}
                                    </TabsTab>
                                ))}
                            </TabsList>
                        </Tabs>
                    </div>
                    <div className="max-w-full overflow-x-auto [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
                        <Tabs value={server ?? "all"} onValueChange={(v) => setSearch({ server: v === "all" ? undefined : (v as string) })}>
                            <TabsList aria-label={t("users.server.label")}>
                                <TabsTab value="all" className={PHONE_TAB}>
                                    {t("users.server.all")}
                                </TabsTab>
                                {PEOPLE_SERVERS.map((s) => (
                                    <TabsTab key={s} value={s} className={PHONE_TAB}>
                                        {s.toUpperCase()}
                                    </TabsTab>
                                ))}
                            </TabsList>
                        </Tabs>
                    </div>
                    {total !== undefined ? <span className="ml-auto whitespace-nowrap text-[12.5px] text-muted-foreground tabular-nums">{t("users.count", { shown: f.number(rows.length), total: f.number(total) })}</span> : null}
                </AdminToolbar>

                {firstPage?.isError ? (
                    <PanelMessage className="p-10">{t("users.error", { reason: describeError(firstPage.error) })}</PanelMessage>
                ) : firstPage?.isPending ? (
                    <PanelMessage className="p-10">{t("users.loading")}</PanelMessage>
                ) : rows.length === 0 ? (
                    <PanelMessage className="p-10">{t("users.empty")}</PanelMessage>
                ) : (
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead className="pl-[18px]">{t("users.col.player")}</TableHead>
                                <TableHead className="max-md:hidden">{t("users.col.server")}</TableHead>
                                <TableHead className="max-md:hidden">{t("users.col.level")}</TableHead>
                                <TableHead className="max-md:hidden">{t("users.col.score")}</TableHead>
                                <TableHead className="max-md:hidden">{t("users.col.grade")}</TableHead>
                                <TableHead className="max-sm:hidden">{t("users.col.role")}</TableHead>
                                <TableHead className="max-sm:hidden">{t("users.col.canEdit")}</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {rows.map((u) => {
                                const held = grants ? grantsOf(grants.byUser, u.id) : undefined;
                                const warned = held ? accessWarning(u.role, held) !== null : false;
                                const text = held ? accessText(t, accessSummary(u.role, held, localeName)) : "";
                                const label = personLabel(u, t("users.deletedAccount"));
                                // Shown in their own columns from sm, under the name on a phone.
                                const role = <Badge variant={roleVariant(u.role)}>{roleLabel(u.role)}</Badge>;
                                const canEdit = <span className={cn("text-[13px]", warned ? "text-warning-foreground" : text === "—" ? "text-muted-foreground" : "text-foreground")}>{warned ? t("users.access.warned", { access: text }) : text}</span>;
                                return (
                                    <TableRow key={u.id} className="cursor-pointer hover:bg-accent" data-state={u.id === selectedId ? "selected" : undefined}>
                                        <TableCell>
                                            {/* The button's ::after covers the row, so the whole row opens the sheet. */}
                                            <button type="button" onClick={() => setSearch({ user: u.id })} className="flex items-center gap-2.5 pl-2 text-left outline-none after:absolute after:inset-0 focus-visible:after:ring-2 focus-visible:after:ring-ring">
                                                <PersonAvatar avatarId={u.avatarId} name={label} />
                                                <span className="flex flex-col gap-[3px]">
                                                    <span className="font-medium text-[13.5px]">{label}</span>
                                                    <span className="whitespace-nowrap font-mono text-[12px] text-muted-foreground leading-[1.2]">{t("users.uid", { uid: u.uid })}</span>
                                                    <span className="flex flex-wrap items-center gap-x-2 gap-y-1 whitespace-normal pt-0.5 leading-snug sm:hidden">
                                                        {role}
                                                        {canEdit}
                                                    </span>
                                                </span>
                                            </button>
                                        </TableCell>
                                        <TableCell className="max-md:hidden">
                                            <span className="font-mono text-[12.5px]">{u.server}</span>
                                        </TableCell>
                                        <TableCell className="tabular-nums max-md:hidden">{u.level === null ? "—" : f.number(u.level)}</TableCell>
                                        <TableCell className="tabular-nums max-md:hidden">{u.totalScore === null ? "—" : f.number(Math.round(u.totalScore))}</TableCell>
                                        <TableCell className="max-md:hidden">{u.grade ? <Badge variant="outline">{u.grade}</Badge> : <span className="text-muted-foreground">—</span>}</TableCell>
                                        <TableCell className="max-sm:hidden">{role}</TableCell>
                                        <TableCell className="min-w-[16ch] max-w-[28ch] whitespace-normal leading-snug max-sm:hidden">{canEdit}</TableCell>
                                    </TableRow>
                                );
                            })}
                        </TableBody>
                    </Table>
                )}

                {total !== undefined && rows.length < total ? (
                    <div className="flex justify-center border-border border-t p-3">
                        <Button size="sm" variant="outline" loading={loadingMore} onClick={() => setPaging({ key: filterKey, pages: pages + 1 })}>
                            {t("users.more")}
                        </Button>
                    </div>
                ) : null}
            </Card>

            <PersonSheet person={person} onClose={() => setSearch({ user: undefined })} meId={me?.id ?? null} canAssign={access.canAssign} grants={grants} locales={locales} localeName={localeName} />
        </>
    );
}
