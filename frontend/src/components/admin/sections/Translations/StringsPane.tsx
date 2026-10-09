import { useInfiniteQuery, useQuery } from "@tanstack/react-query";
import { ArrowLeftIcon, SearchIcon } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import type { AdminTranslationsFilter, IAdminTranslationsSearch } from "#/components/admin/shell/search";
import { useScrollTopOnOpen } from "#/components/admin/shell/useScrollTopOnOpen";
import { Button } from "#/components/ui/button";
import { Card } from "#/components/ui/card";
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyTitle } from "#/components/ui/empty";
import { InputGroup, InputGroupAddon, InputGroupInput } from "#/components/ui/input-group";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Skeleton } from "#/components/ui/skeleton";
import { useAuth } from "#/hooks/use-auth";
import { useDebounce } from "#/hooks/use-debounce";
import { listTranslationsFn, translationsListQueryOptions } from "#/lib/api/admin";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { Locale } from "#/types/generated/Locale";
import { Editor } from "./Editor";
import { KeyList } from "./KeyList";
import { flattenPages, type ILocaleStats, keyAfterSave, keyAfterSkip, selectEntry } from "./model";
import type { messages } from "./Translations.messages";

type T = TypedT<typeof messages>;

/** The backend's page size for `/admin/i18n/messages`. */
const PAGE_SIZE = 40;
const SEARCH_DEBOUNCE_MS = 260;
/** The Area select's value for "every namespace"; the URL carries it as no `ns` at all. */
const ALL_AREAS = "*";

interface IStringsPaneProps {
    search: IAdminTranslationsSearch;
    locale: Locale;
    canEdit: boolean;
    filter: AdminTranslationsFilter;
    stats: ILocaleStats;
    statsReady: boolean;
    namespaces: string[] | undefined;
    enabled: boolean;
    setSearch: (patch: Partial<IAdminTranslationsSearch>, replace?: boolean) => void;
}

export function StringsPane({ search, locale, canEdit, filter, stats, statsReady, namespaces, enabled, setSearch }: IStringsPaneProps): React.ReactElement {
    const t: T = useT("admin");
    const fmt = useFormatters();
    const { isAuthenticated } = useAuth();

    // The box is local so typing stays instant; the debounced value goes to the URL, which drives the query.
    const [qInput, setQInput] = useState(search.q ?? "");
    const q = useDebounce(qInput.trim(), SEARCH_DEBOUNCE_MS);
    const pushedQ = useRef(search.q ?? "");
    useEffect(() => {
        const external = search.q ?? "";
        if (external === pushedQ.current) return;
        pushedQ.current = external;
        setQInput(external);
    }, [search.q]);
    useEffect(() => {
        if (q === pushedQ.current) return;
        pushedQ.current = q;
        setSearch({ q: q || undefined });
    }, [q, setSearch]);

    const listInput = { locale: locale.code, namespace: search.ns, search: search.q, filter };
    const listQuery = useInfiniteQuery({
        // Under `["admin", "i18n", "messages"]`, so every write's invalidation reaches it.
        queryKey: ["admin", "i18n", "messages", "pages", listInput, isAuthenticated ? "auth" : "anon"] as const,
        queryFn: ({ pageParam }) => listTranslationsFn({ data: { ...listInput, limit: PAGE_SIZE, offset: pageParam } }),
        initialPageParam: 0,
        getNextPageParam: (last, pages) => {
            const loaded = pages.reduce((n, p) => n + p.entries.length, 0);
            return last.entries.length > 0 && loaded < last.total ? loaded : undefined;
        },
        // Typing a search keeps the old rows up until the new ones land, instead of flashing skeletons.
        placeholderData: (previous) => previous,
        enabled,
        staleTime: 15 * 1000,
        gcTime: 5 * 60 * 1000,
    });
    const entries = useMemo(() => flattenPages(listQuery.data?.pages ?? []), [listQuery.data]);
    const total = listQuery.data?.pages[0]?.total ?? 0;

    // A deep link can name a key that is not on a loaded page (or not in this filter at all): fetch it by name.
    const listedIndex = search.key ? entries.findIndex((e) => e.key === search.key) : -1;
    const needsLookup = search.key !== undefined && listQuery.isSuccess && !listQuery.isPlaceholderData && listedIndex < 0;
    const lookupQuery = useQuery({ ...translationsListQueryOptions({ locale: locale.code, search: search.key, filter: "all", limit: 20 }, isAuthenticated), enabled: enabled && needsLookup });
    const lookedUp = needsLookup ? lookupQuery.data?.entries.find((e) => e.key === search.key) : undefined;
    const lookupPending = needsLookup && lookupQuery.isPending;

    const selected = lookedUp ?? (lookupPending ? undefined : selectEntry(entries, search.key));
    const selectedIndex = selected ? entries.findIndex((e) => e.key === selected.key) : -1;

    // Below lg the URL's key picks the pane: the editor while a string is open, the list otherwise.
    const detail = search.key !== undefined;
    const go = (key: string | undefined) => setSearch({ key });
    const back = () => setSearch({ key: undefined }, false);
    // Opening a string from the stacked layout swaps the list out for the editor; start the page at
    // its top.
    const { listRef, arm: armScrollTop } = useScrollTopOnOpen(search.key);
    const select = (key: string) => {
        armScrollTop();
        setSearch({ key }, false);
    };

    const narrowed = Boolean(search.q || search.ns);
    const showAll = () => {
        setSearch({ filter: "all", ns: undefined, key: undefined });
    };
    const resetSearch = () => {
        setQInput("");
        setSearch({ q: undefined, ns: undefined, key: undefined });
    };

    const filters: { id: AdminTranslationsFilter; label: string; count: number | null }[] = [
        { id: "todo", label: t("translations.filter.todo"), count: statsReady ? stats.todo : null },
        { id: "stale", label: t("translations.filter.stale"), count: statsReady ? stats.stale : null },
        { id: "untranslated", label: t("translations.filter.untranslated"), count: statsReady ? stats.missing : null },
        { id: "all", label: t("translations.filter.all"), count: null },
    ];
    const current = filters.find((f) => f.id === filter) ?? filters[0];
    const sortedNamespaces = useMemo(() => [...(namespaces ?? [])].sort((a, b) => a.localeCompare(b)), [namespaces]);

    return (
        <div className="grid grid-cols-[minmax(0,1fr)] items-start gap-4 lg:grid-cols-[320px_minmax(0,1fr)] xl:grid-cols-[400px_minmax(0,1fr)]">
            <Card ref={listRef} className={cn("overflow-hidden lg:sticky lg:top-20 lg:h-[calc(100dvh-6rem)]", detail && "max-lg:hidden")}>
                <div className="@container flex flex-wrap gap-2 border-border border-b p-3">
                    <InputGroup className="min-w-0 flex-[1_1_5rem]">
                        <InputGroupAddon>
                            <SearchIcon />
                        </InputGroupAddon>
                        <InputGroupInput size="sm" type="search" value={qInput} onChange={(e) => setQInput(e.target.value)} placeholder={t("translations.list.search")} aria-label={t("translations.list.searchLabel")} />
                    </InputGroup>
                    <Select value={filter} onValueChange={(v) => v && setSearch({ filter: v as AdminTranslationsFilter, key: undefined })}>
                        <SelectTrigger size="sm" className="w-auto min-w-0 shrink-0" aria-label={t("translations.filter.label")}>
                            <SelectValue>
                                {() => (
                                    <>
                                        {current.label}
                                        {/* The count only rides in the trigger when the header has room; the options always carry it. */}
                                        {current.count !== null ? <span className="ms-1.5 @[22rem]:inline hidden text-muted-foreground tabular-nums">{fmt.number(current.count)}</span> : null}
                                    </>
                                )}
                            </SelectValue>
                        </SelectTrigger>
                        <SelectContent>
                            {filters.map((f) => (
                                <SelectItem key={f.id} value={f.id} className={cn(f.count === 0 && "text-muted-foreground")}>
                                    <span className="flex items-center gap-4">
                                        {f.label}
                                        {f.count !== null ? <span className="ms-auto text-muted-foreground tabular-nums">{fmt.number(f.count)}</span> : null}
                                    </span>
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                    <Select value={search.ns ?? ALL_AREAS} onValueChange={(v) => v && setSearch({ ns: v === ALL_AREAS ? undefined : v, key: undefined })}>
                        <SelectTrigger size="sm" className="w-auto min-w-0 max-w-36 shrink" aria-label={t("translations.area.label")}>
                            <SelectValue>{() => search.ns ?? t("translations.area.all")}</SelectValue>
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value={ALL_AREAS}>{t("translations.area.all")}</SelectItem>
                            {sortedNamespaces.map((ns) => (
                                <SelectItem key={ns} value={ns}>
                                    {ns}
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                </div>
                <KeyList
                    entries={entries}
                    selectedKey={selectedIndex >= 0 ? selected?.key : undefined}
                    pending={listQuery.isPending}
                    error={listQuery.isError && entries.length === 0}
                    onRetry={() => void listQuery.refetch()}
                    emptyText={filter === "all" || narrowed ? t("translations.list.emptyAll") : t("translations.list.emptyFiltered", { language: locale.english_name })}
                    hasMore={listQuery.hasNextPage}
                    loadingMore={listQuery.isFetchingNextPage}
                    onLoadMore={() => void listQuery.fetchNextPage()}
                    total={total}
                    onSelect={select}
                />
            </Card>

            <div className={cn("min-w-0", !detail && "max-lg:hidden")}>
                {selected ? (
                    <Editor key={`${locale.code}:${selected.key}`} entry={selected} locale={locale} canEdit={canEdit} position={selectedIndex >= 0 ? { index: selectedIndex, total } : null} nextKey={keyAfterSave(entries, selected.key)} skipKey={keyAfterSkip(entries, selected.key)} onGo={go} onBack={back} />
                ) : listQuery.isPending || lookupPending ? (
                    <Skeleton className="h-[480px] rounded-2xl" />
                ) : (
                    <Card>
                        <div className="px-3 pt-3 lg:hidden">
                            <Button size="sm" variant="ghost" onClick={back}>
                                <ArrowLeftIcon />
                                {t("translations.editor.back")}
                            </Button>
                        </div>
                        {filter === "all" || narrowed ? (
                            <Empty>
                                <EmptyHeader>
                                    <EmptyTitle>{t("translations.noMatch.title")}</EmptyTitle>
                                    <EmptyDescription>{t("translations.noMatch.desc")}</EmptyDescription>
                                </EmptyHeader>
                                {narrowed ? (
                                    <EmptyContent>
                                        <Button size="sm" variant="outline" onClick={resetSearch}>
                                            {t("translations.noMatch.reset")}
                                        </Button>
                                    </EmptyContent>
                                ) : null}
                            </Empty>
                        ) : (
                            <Empty>
                                <EmptyHeader>
                                    <EmptyTitle>{t("translations.done.title", { language: locale.english_name })}</EmptyTitle>
                                    <EmptyDescription>{t("translations.done.desc")}</EmptyDescription>
                                </EmptyHeader>
                                <EmptyContent>
                                    <Button size="sm" variant="outline" onClick={showAll}>
                                        {t("translations.done.showAll")}
                                    </Button>
                                </EmptyContent>
                            </Empty>
                        )}
                    </Card>
                )}
            </div>
        </div>
    );
}
