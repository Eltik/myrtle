import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { CheckIcon, EraserIcon, HistoryIcon, LanguagesIcon, PlusIcon, RefreshCwIcon, RotateCcwIcon, SearchIcon, Trash2Icon, XIcon } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { Pagination } from "#/components/operators/list/impl/components/Pagination";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from "#/components/ui/card";
import { FilterChip } from "#/components/ui/filter-chip";
import { Input } from "#/components/ui/input";
import { InputGroup, InputGroupAddon } from "#/components/ui/input-group";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "#/components/ui/menu";
import { Skeleton } from "#/components/ui/skeleton";
import { Textarea } from "#/components/ui/textarea";
import { toastManager } from "#/components/ui/toast";
import { useAuth } from "#/hooks/use-auth";
import { useDebounce } from "#/hooks/use-debounce";
import {
    clearTranslationFn,
    grantTranslationPermissionFn,
    type IGrantTranslationPermissionInput,
    type IListTranslationsInput,
    type ITranslationFieldError,
    type IUpdateTranslationInput,
    isSuperAdmin,
    localesQueryOptions,
    revokeTranslationPermissionFn,
    type TierListPermissionLevel,
    type TranslationFilter,
    translationEntryAuditQueryOptions,
    translationNamespacesQueryOptions,
    translationPermissionsQueryOptions,
    translationProgressQueryOptions,
    translationsListQueryOptions,
    updateTranslationFn,
    writableLocalesQueryOptions,
} from "#/lib/api/admin";
import { searchUsersQueryOptions } from "#/lib/api/user";
import { type TypedRichT, useFormatters, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, getSecretaryAvatarURL } from "#/lib/utils";
import type { JsonValue } from "#/types/generated/serde_json/JsonValue";
import type { TranslationEntry } from "#/types/generated/TranslationEntry";
import { HCode, PageHead } from "../AdminShell";
import { MonoSection } from "../Primitives";
import type { messages as primitivesMessages } from "../Primitives.messages";
import { LocalesSection } from "./Locales";
import type { messages } from "./Translations.messages";

/** The grant rung names are the ones `LevelBadge` declares. */
type TranslationsT = TypedT<typeof messages & typeof primitivesMessages>;
type TranslationsRichT = TypedRichT<typeof messages & typeof primitivesMessages>;

const PAGE_SIZE = 40;
const SEARCH_DEBOUNCE_MS = 260;

const FILTERS: readonly TranslationFilter[] = ["all", "untranslated", "stale", "translated"] as const;

function filterLabel(t: TranslationsT, filter: TranslationFilter): string {
    switch (filter) {
        case "all":
            return t("i18n.filter.all");
        case "untranslated":
            return t("i18n.filter.untranslated");
        case "stale":
            return t("i18n.filter.stale");
        case "translated":
            return t("i18n.filter.translated");
    }
}

const GRANT_LEVELS: readonly TierListPermissionLevel[] = ["view", "edit", "publish", "admin"] as const;

function levelLabel(t: TranslationsT, level: string): string {
    switch (level) {
        case "view":
            return t("level.view");
        case "edit":
            return t("level.edit");
        case "publish":
            return t("level.publish");
        case "admin":
            return t("level.admin");
        default:
            return level;
    }
}

/**
 * `ui_message_keys.placeholders` is a JSON array of argument names. Objects are
 * tolerated so an older row shape never blanks the chip row.
 */
function placeholderNames(raw: JsonValue): string[] {
    if (Array.isArray(raw)) return raw.filter((v): v is string => typeof v === "string");
    if (raw !== null && typeof raw === "object") return Object.keys(raw);
    return [];
}

function entryState(entry: TranslationEntry): "untranslated" | "stale" | "translated" {
    if (entry.value === null) return "untranslated";
    return entry.is_stale ? "stale" : "translated";
}

export function Translations(): React.ReactElement {
    const t: TranslationsT = useT("admin");
    const rt: TranslationsRichT = useRichT("admin");
    const fmt = useFormatters();
    const { user, isAuthenticated } = useAuth();
    const localesQuery = useQuery(localesQueryOptions(isAuthenticated));
    const writableQuery = useQuery(writableLocalesQueryOptions(isAuthenticated));
    const progressQuery = useQuery(translationProgressQueryOptions(isAuthenticated));
    const namespacesQuery = useQuery(translationNamespacesQueryOptions(isAuthenticated));

    const [locale, setLocale] = useState<string>("");
    const [filter, setFilter] = useState<TranslationFilter>("all");
    const [namespace, setNamespace] = useState<string>("");
    const [searchInput, setSearchInput] = useState<string>("");
    const [search, setSearch] = useState<string>("");
    const [page, setPage] = useState(1);
    const [selectedKey, setSelectedKey] = useState<string | null>(null);
    const [historyKey, setHistoryKey] = useState<string | null>(null);
    const [showGrant, setShowGrant] = useState(false);

    const locales = localesQuery.data ?? [];
    const writable = writableQuery.data ?? [];

    // A tier-list admin can read progress without holding a single grant, so the
    // picker falls back to every locale and the editor goes read-only.
    const pickable = useMemo(() => (writable.length > 0 ? locales.filter((l) => writable.includes(l.code)) : locales), [locales, writable]);

    useEffect(() => {
        if (locale || pickable.length === 0) return;
        setLocale(pickable[0].code);
    }, [locale, pickable]);

    useEffect(() => {
        const id = window.setTimeout(() => setSearch(searchInput.trim()), SEARCH_DEBOUNCE_MS);
        return () => window.clearTimeout(id);
    }, [searchInput]);

    // Any change to what is being listed resets the pager and the open editor.
    //
    // The dependency array was empty, so this only ran on mount and the offset
    // survived every filter change: switching to `untranslated` from page 30 of
    // `all` left the offset at 30 and rendered an empty "no match" page for a
    // filter with thousands of rows. That reads exactly like the list failing
    // to load until you page further.
    // biome-ignore lint/correctness/useExhaustiveDependencies: the setters are the effect's body, not inputs; these four are what "what is being listed" means
    useEffect(() => {
        setPage(1);
        setSelectedKey(null);
    }, [locale, filter, namespace, search]);

    const listInput: IListTranslationsInput = {
        locale,
        namespace: namespace || undefined,
        search: search || undefined,
        filter,
        limit: PAGE_SIZE,
        offset: (page - 1) * PAGE_SIZE,
    };
    const listQuery = useQuery(translationsListQueryOptions(listInput, isAuthenticated));

    // The namespace list is a property of the catalog, not of the open page, so
    // it comes from its own endpoint - the options are the same whichever page
    // or locale is being read.
    const namespaces = useMemo(() => [...(namespacesQuery.data ?? [])].sort((a, b) => a.localeCompare(b)), [namespacesQuery.data]);

    const canWrite = writable.includes(locale);
    const entries = listQuery.data?.entries ?? [];
    const total = listQuery.data?.total ?? 0;
    const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE));

    const progress = progressQuery.data ?? [];
    const localeProgress = progress.find((p) => p.locale === locale);
    const counts: Record<TranslationFilter, number | undefined> = localeProgress
        ? {
              all: localeProgress.total,
              untranslated: localeProgress.total - localeProgress.translated,
              stale: localeProgress.stale,
              translated: localeProgress.translated - localeProgress.stale,
          }
        : { all: undefined, untranslated: undefined, stale: undefined, translated: undefined };

    const selected = selectedKey ? (entries.find((e) => e.key === selectedKey) ?? null) : null;
    const localeMeta = locales.find((l) => l.code === locale) ?? null;

    return (
        <>
            <PageHead
                kicker={t("i18n.kicker")}
                title={t("i18n.title")}
                sub={rt("i18n.sub", { table: <HCode>ui_messages</HCode>, auditTable: <HCode>ui_message_audit</HCode> })}
                action={
                    <Button
                        variant="outline"
                        size="sm"
                        loading={listQuery.isFetching || progressQuery.isFetching}
                        onClick={() => {
                            void listQuery.refetch();
                            void progressQuery.refetch();
                        }}
                    >
                        <RefreshCwIcon />
                        {t("i18n.refresh")}
                    </Button>
                }
            />

            <ProgressStrip pending={progressQuery.isPending} rows={progress} locales={locales} active={locale} onSelect={(code) => setLocale(code)} />

            <div className="h-4" />

            <div className="relative mb-4 overflow-hidden rounded-2xl border border-border bg-card shadow-xs/5 before:pointer-events-none before:absolute before:inset-0 before:rounded-[calc(var(--radius-2xl)-1px)] before:shadow-[0_1px_--theme(--color-black/4%)] dark:before:shadow-[0_-1px_--theme(--color-white/6%)]">
                <div className="flex flex-wrap items-center gap-2.5 border-border border-b p-3.5">
                    <DropdownMenu>
                        <DropdownMenuTrigger
                            render={(triggerProps) => (
                                <button {...triggerProps} type="button" className="inline-flex h-7 shrink-0 cursor-pointer items-center gap-1.5 rounded-md border border-border bg-card px-2.5 font-medium text-[12.5px] text-foreground hover:bg-accent">
                                    <LanguagesIcon className="size-3.5 opacity-70" strokeWidth={1.9} />
                                    <span className="font-mono">{locale || "-"}</span>
                                    {localeMeta ? <span className="text-muted-foreground">· {localeMeta.native_name}</span> : null}
                                </button>
                            )}
                        />
                        <DropdownMenuContent align="start" className="w-64">
                            {pickable.length === 0 ? (
                                <DropdownMenuItem disabled>{t("i18n.noLocales")}</DropdownMenuItem>
                            ) : (
                                pickable.map((l) => (
                                    <DropdownMenuItem key={l.code} className="cursor-pointer" onClick={() => setLocale(l.code)}>
                                        {locale === l.code ? <CheckIcon className="mr-2 h-4 w-4 text-primary" /> : <span className="mr-2 inline-block h-4 w-4" />}
                                        <span className="font-mono text-[12px]">{l.code}</span>
                                        <span className="ml-1.5 text-muted-foreground">{l.native_name}</span>
                                        {!writable.includes(l.code) ? <span className="ml-auto font-mono text-[10.5px] text-muted-foreground">{t("i18n.readOnlyTag")}</span> : null}
                                    </DropdownMenuItem>
                                ))
                            )}
                        </DropdownMenuContent>
                    </DropdownMenu>

                    <DropdownMenu>
                        <DropdownMenuTrigger
                            render={(triggerProps) => (
                                <button {...triggerProps} type="button" className="inline-flex h-7 shrink-0 cursor-pointer items-center gap-1.5 rounded-md border border-border bg-card px-2.5 font-medium text-[12.5px] text-foreground hover:bg-accent">
                                    <span className="text-muted-foreground">{t("i18n.nsLabel")}</span>
                                    <span className="font-mono">{namespace || t("i18n.nsAll")}</span>
                                </button>
                            )}
                        />
                        <DropdownMenuContent align="start" className="w-56">
                            <DropdownMenuItem className="cursor-pointer" onClick={() => setNamespace("")}>
                                {namespace === "" ? <CheckIcon className="mr-2 h-4 w-4 text-primary" /> : <span className="mr-2 inline-block h-4 w-4" />}
                                {t("i18n.allNamespaces")}
                            </DropdownMenuItem>
                            {namespaces.map((ns) => (
                                <DropdownMenuItem key={ns} className="cursor-pointer" onClick={() => setNamespace(ns)}>
                                    {namespace === ns ? <CheckIcon className="mr-2 h-4 w-4 text-primary" /> : <span className="mr-2 inline-block h-4 w-4" />}
                                    <span className="font-mono text-[12px]">{ns}</span>
                                </DropdownMenuItem>
                            ))}
                        </DropdownMenuContent>
                    </DropdownMenu>

                    <div className="w-full min-w-0 max-w-95 sm:min-w-60 sm:flex-1">
                        <InputGroup>
                            <InputGroupAddon>
                                <SearchIcon />
                            </InputGroupAddon>
                            <Input placeholder={t("i18n.searchPlaceholder")} size="sm" value={searchInput} onChange={(e) => setSearchInput(e.target.value)} />
                        </InputGroup>
                    </div>

                    <div className="flex-1" />
                    <span className="font-mono text-[11.5px] text-muted-foreground tabular-nums">{t("i18n.keyCount", { count: fmt.number(total) })}</span>
                </div>

                <div className="flex flex-wrap items-center gap-2 border-border border-b p-3.5">
                    {FILTERS.map((option) => (
                        <FilterChip key={option} label={filterLabel(t, option)} active={filter === option} count={counts[option]} onSelect={() => setFilter(option)} />
                    ))}
                    {!canWrite && locale ? <span className="ml-auto text-[12px] text-muted-foreground">{rt("i18n.noGrant", { locale: <span className="font-mono">{locale}</span> })}</span> : null}
                </div>

                {listQuery.isError ? (
                    <div className="px-3.5 py-16 text-center text-[13px] text-muted-foreground">{t("i18n.loadError")}</div>
                ) : listQuery.isPending ? (
                    <div className="space-y-2 p-4">
                        <Skeleton className="h-16" />
                        <Skeleton className="h-16" />
                        <Skeleton className="h-16" />
                    </div>
                ) : entries.length === 0 ? (
                    <div className="px-3.5 py-16 text-center text-[13px] text-muted-foreground">
                        {search || namespace || filter !== "all" ? (
                            <>
                                {t("i18n.noMatch")}{" "}
                                <button
                                    type="button"
                                    className="cursor-pointer text-foreground underline underline-offset-2 hover:text-primary"
                                    onClick={() => {
                                        setSearchInput("");
                                        setNamespace("");
                                        setFilter("all");
                                    }}
                                >
                                    {t("i18n.reset")}
                                </button>
                            </>
                        ) : (
                            t("i18n.empty")
                        )}
                    </div>
                ) : (
                    <ul className="divide-y divide-border">
                        {entries.map((entry) => (
                            <li key={entry.key}>
                                <MessageRow entry={entry} active={selectedKey === entry.key} onOpen={() => setSelectedKey(entry.key)} />
                            </li>
                        ))}
                    </ul>
                )}

                {totalPages > 1 ? (
                    <div className="border-border border-t px-3.5 pt-1 pb-3.5">
                        <Pagination currentPage={page} totalPages={totalPages} onPageChange={setPage} className="mt-2" />
                    </div>
                ) : null}
            </div>

            {selected ? <MessageEditor key={`${locale}:${selected.key}`} locale={locale} entry={selected} canWrite={canWrite} onClose={() => setSelectedKey(null)} onOpenHistory={() => setHistoryKey(selected.key)} /> : null}

            <div className="h-4" />

            <GrantsSection locale={locale} userId={user?.id ?? null} role={user?.role ?? null} authed={isAuthenticated} onGrant={() => setShowGrant(true)} />

            <div className="h-4" />

            <LocalesSection role={user?.role ?? null} authed={isAuthenticated} />

            {historyKey ? <HistoryDrawer locale={locale} messageKey={historyKey} canWrite={canWrite} authed={isAuthenticated} onClose={() => setHistoryKey(null)} /> : null}
            {showGrant && locale ? <GrantDialog locale={locale} onClose={() => setShowGrant(false)} /> : null}
        </>
    );
}

// ---------------------------------------------------------------- progress

function ProgressStrip({ pending, rows, locales, active, onSelect }: { pending: boolean; rows: { locale: string; total: number; translated: number; stale: number }[]; locales: { code: string; native_name: string }[]; active: string; onSelect: (code: string) => void }): React.ReactElement {
    const t: TranslationsT = useT("admin");
    const fmt = useFormatters();
    if (pending) {
        return (
            <div className="grid grid-cols-2 gap-3 sm:gap-3.5 lg:grid-cols-4">
                <Skeleton className="h-24 rounded-2xl" />
                <Skeleton className="h-24 rounded-2xl" />
                <Skeleton className="h-24 rounded-2xl" />
                <Skeleton className="h-24 rounded-2xl" />
            </div>
        );
    }
    if (rows.length === 0) {
        return <div className="rounded-2xl border border-border bg-card px-3.5 py-8 text-center text-[13px] text-muted-foreground">{t("i18n.progress.empty")}</div>;
    }
    return (
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 sm:gap-3.5 lg:grid-cols-4">
            {rows.map((p) => {
                const pct = p.total > 0 ? Math.round((p.translated / p.total) * 100) : 0;
                const untranslated = p.total - p.translated;
                const meta = locales.find((l) => l.code === p.locale);
                return (
                    <button
                        key={p.locale}
                        type="button"
                        onClick={() => onSelect(p.locale)}
                        className={cn(
                            "relative min-w-0 cursor-pointer rounded-2xl border bg-card p-3.5 text-left shadow-xs/5 transition-colors before:pointer-events-none before:absolute before:inset-0 before:rounded-[calc(var(--radius-2xl)-1px)] before:shadow-[0_1px_--theme(--color-black/4%)] dark:before:shadow-[0_-1px_--theme(--color-white/6%)]",
                            active === p.locale ? "border-primary/40 bg-[color-mix(in_srgb,var(--primary)_5%,var(--card))]" : "border-border hover:bg-[color-mix(in_srgb,var(--card),oklch(0_0_0)_2%)]",
                        )}
                    >
                        <div className="flex items-baseline justify-between gap-2">
                            <span className="font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em]">{p.locale}</span>
                            <span className="truncate text-[11.5px] text-muted-foreground">{meta?.native_name ?? ""}</span>
                        </div>
                        <div className="mt-2 flex flex-wrap items-baseline gap-x-1 font-bold text-[20px] tabular-nums leading-none tracking-[-0.02em]">
                            <span>{pct}%</span>
                            <span className="font-medium font-mono text-[11px] text-muted-foreground">{t("i18n.progress.ratio", { translated: fmt.number(p.translated), total: fmt.number(p.total) })}</span>
                        </div>
                        <span className="mt-2.5 block h-1.5 overflow-hidden rounded-[3px] bg-muted">
                            <span className="block h-full bg-success-foreground/70" style={{ width: `${pct}%` }} />
                        </span>
                        <div className="mt-2 flex flex-wrap items-center gap-1.5">
                            {p.stale > 0 ? <Badge variant="warning">{t("i18n.progress.stale", { count: fmt.number(p.stale) })}</Badge> : null}
                            {untranslated > 0 ? <Badge variant="outline">{t("i18n.progress.untranslated", { count: fmt.number(untranslated) })}</Badge> : null}
                            {p.stale === 0 && untranslated === 0 ? <Badge variant="success">{t("i18n.progress.complete")}</Badge> : null}
                        </div>
                    </button>
                );
            })}
        </div>
    );
}

// ---------------------------------------------------------------- list row

function MessageRow({ entry, active, onOpen }: { entry: TranslationEntry; active: boolean; onOpen: () => void }): React.ReactElement {
    const t: TranslationsT = useT("admin");
    const fmt = useFormatters();
    const state = entryState(entry);
    return (
        <button type="button" onClick={onOpen} className={cn("group flex w-full min-w-0 cursor-pointer items-start gap-3 px-3.5 py-3 text-left transition-colors hover:bg-[color-mix(in_srgb,var(--card),oklch(0_0_0)_2%)] sm:px-4", active && "bg-[color-mix(in_srgb,var(--primary)_6%,var(--card))]")}>
            <span aria-hidden className={cn("mt-1.5 size-1.75 shrink-0 rounded-full", state === "translated" && "bg-success-foreground/70", state === "stale" && "bg-warning-foreground/80", state === "untranslated" && "bg-muted-foreground/40")} />
            <div className="flex min-w-0 flex-1 flex-col gap-1">
                <div className="flex min-w-0 flex-wrap items-baseline gap-x-2 gap-y-0.5">
                    <span className="truncate font-mono text-[11px] text-muted-foreground">{entry.key}</span>
                    <span className="inline-flex h-4 items-center rounded-sm border border-border bg-muted px-1 font-mono text-[10px] text-muted-foreground leading-none">{entry.namespace}</span>
                </div>
                <p className="line-clamp-2 text-[13px] leading-snug">{entry.source_text}</p>
                {entry.value !== null ? <p className="line-clamp-2 text-[12.5px] text-muted-foreground leading-snug">{entry.value}</p> : <p className="text-[12px] text-muted-foreground/70 italic">{t("i18n.notTranslated")}</p>}
            </div>
            <div className="flex shrink-0 flex-col items-end gap-1.5">
                {state === "stale" ? <Badge variant="warning">{t("i18n.badge.stale")}</Badge> : state === "untranslated" ? <Badge variant="outline">{t("i18n.badge.untranslated")}</Badge> : <Badge variant="success">{t("i18n.badge.translated")}</Badge>}
                {entry.updated_at ? <span className="whitespace-nowrap font-mono text-[10.5px] text-muted-foreground">{fmt.relativeShort(entry.updated_at)}</span> : null}
            </div>
        </button>
    );
}

// ---------------------------------------------------------------- editor

function MessageEditor({ locale, entry, canWrite, onClose, onOpenHistory }: { locale: string; entry: TranslationEntry; canWrite: boolean; onClose: () => void; onOpenHistory: () => void }): React.ReactElement {
    const t: TranslationsT = useT("admin");
    const fmt = useFormatters();
    const queryClient = useQueryClient();
    const [value, setValue] = useState<string>(entry.value ?? "");
    const [baseline, setBaseline] = useState<string>(entry.value ?? "");
    const [fieldErrors, setFieldErrors] = useState<ITranslationFieldError[]>([]);

    const placeholders = useMemo(() => placeholderNames(entry.placeholders), [entry.placeholders]);
    const dirty = value !== baseline;
    const state = entryState(entry);

    const invalidate = () => {
        void queryClient.invalidateQueries({ queryKey: ["admin", "i18n", "messages"] });
        void queryClient.invalidateQueries({ queryKey: ["admin", "i18n", "progress"] });
        void queryClient.invalidateQueries({ queryKey: ["admin", "i18n", "audit"] });
    };

    const save = useMutation({
        mutationFn: (input: IUpdateTranslationInput) => updateTranslationFn({ data: input }),
        onSuccess: (result) => {
            if (!result.ok) {
                setFieldErrors(result.details);
                return;
            }
            setFieldErrors([]);
            setBaseline(result.entry.value ?? "");
            invalidate();
            toastManager.add({ id: `i18n-save-${Date.now()}`, title: t("i18n.toast.saved"), description: t("i18n.toast.saved.desc", { key: entry.key, locale }), type: "success" });
        },
        onError: (err: unknown) => toastManager.add({ id: `i18n-save-err-${Date.now()}`, title: t("i18n.toast.saveFailed"), description: err instanceof Error ? err.message : String(err), type: "error" }),
    });

    const clear = useMutation({
        mutationFn: () => clearTranslationFn({ data: { locale, key: entry.key } }),
        onSuccess: () => {
            setValue("");
            setBaseline("");
            setFieldErrors([]);
            invalidate();
            toastManager.add({ id: `i18n-clear-${Date.now()}`, title: t("i18n.toast.cleared"), description: t("i18n.toast.cleared.desc", { key: entry.key }), type: "success" });
        },
        onError: (err: unknown) => toastManager.add({ id: `i18n-clear-err-${Date.now()}`, title: t("i18n.toast.clearFailed"), description: err instanceof Error ? err.message : String(err), type: "error" }),
    });

    const invalid = fieldErrors.length > 0;

    return (
        <Card>
            <CardHeader>
                <CardTitle className="flex min-w-0 flex-wrap items-center gap-2 text-sm">
                    <span className="truncate font-mono text-[12.5px]">{entry.key}</span>
                    <span className="inline-flex h-4.5 items-center rounded-sm border border-border bg-muted px-1.5 font-mono text-[10.5px] text-muted-foreground leading-none">{entry.namespace}</span>
                    {state === "stale" ? <Badge variant="warning">{t("i18n.badge.stale")}</Badge> : null}
                </CardTitle>
                <CardDescription className="text-xs">{state === "stale" ? t("i18n.editor.staleNote") : entry.updated_at ? t("i18n.editor.lastSaved", { when: fmt.relativeShort(entry.updated_at) }) : t("i18n.editor.never")}</CardDescription>
                <CardAction>
                    <div className="flex items-center gap-2">
                        <Button variant="ghost" size="sm" onClick={onOpenHistory}>
                            <HistoryIcon />
                            {t("i18n.history")}
                        </Button>
                        <Button variant="ghost" size="sm" onClick={onClose}>
                            {t("i18n.close")}
                        </Button>
                        <Button size="sm" disabled={!canWrite || !dirty || save.isPending} loading={save.isPending} onClick={() => save.mutate({ locale, key: entry.key, value })}>
                            <CheckIcon />
                            {t("i18n.save")}
                        </Button>
                    </div>
                </CardAction>
            </CardHeader>
            <div className="grid grid-cols-1 border-border border-t lg:grid-cols-[minmax(0,1fr)_320px]">
                <div className="border-border border-b p-4 lg:border-r lg:border-b-0">
                    <div className="mb-1 flex items-center justify-between gap-2">
                        <span className="font-medium text-[12px]">
                            {t("i18n.editor.translationLabel")} <span className="font-mono text-muted-foreground">{locale}</span>
                        </span>
                        {dirty ? <Badge variant="warning">{t("i18n.unsaved")}</Badge> : <Badge variant="outline">{t("i18n.saved")}</Badge>}
                    </div>
                    <Textarea
                        aria-invalid={invalid || undefined}
                        disabled={!canWrite}
                        rows={6}
                        size="sm"
                        value={value}
                        onChange={(e) => {
                            setValue(e.target.value);
                            if (fieldErrors.length > 0) setFieldErrors([]);
                        }}
                        placeholder={canWrite ? t("i18n.editor.placeholder") : t("i18n.editor.placeholderReadOnly")}
                    />
                    {invalid ? (
                        <ul className="mt-1.5 flex flex-col gap-1">
                            {fieldErrors.map((fe) => (
                                <li key={`${fe.field}-${fe.message}`} className="rounded-lg border border-destructive/32 bg-destructive/8 px-2 py-1.5 text-[12px] text-destructive-foreground leading-snug">
                                    <span className="font-mono text-[11px] opacity-75">{fe.field}</span> · {fe.message}
                                </li>
                            ))}
                        </ul>
                    ) : (
                        <p className="mt-1.5 text-[11px] text-muted-foreground">{t("i18n.editor.placeholderRule")}</p>
                    )}

                    <div className="mt-3 flex flex-wrap items-center gap-2">
                        <Button variant="destructive-outline" size="xs" disabled={!canWrite || entry.value === null || clear.isPending} loading={clear.isPending} onClick={() => clear.mutate()}>
                            <EraserIcon />
                            {t("i18n.clear")}
                        </Button>
                        {dirty ? (
                            <Button
                                variant="outline"
                                size="xs"
                                onClick={() => {
                                    setValue(baseline);
                                    setFieldErrors([]);
                                }}
                            >
                                <RotateCcwIcon />
                                {t("i18n.discard")}
                            </Button>
                        ) : null}
                    </div>
                </div>

                <div className="p-4">
                    <MonoSection>{t("i18n.englishSource")}</MonoSection>
                    <p className="mt-1.5 whitespace-pre-line rounded-lg border border-border bg-[color-mix(in_srgb,var(--card),oklch(0_0_0)_2%)] p-2.5 text-[13px] leading-[1.55]">{entry.source_text}</p>

                    {entry.description ? (
                        <>
                            <div className="mt-3.5">
                                <MonoSection>{t("i18n.context")}</MonoSection>
                            </div>
                            <p className="mt-1.5 text-[12.5px] text-muted-foreground leading-[1.55]">{entry.description}</p>
                        </>
                    ) : null}

                    <div className="mt-3.5">
                        <MonoSection>{t("i18n.placeholders")}</MonoSection>
                    </div>
                    {placeholders.length === 0 ? (
                        <p className="mt-1.5 text-[12px] text-muted-foreground italic">{t("i18n.placeholders.none")}</p>
                    ) : (
                        <div className="mt-1.5 flex flex-wrap gap-1.5">
                            {placeholders.map((name) => {
                                const present = value.includes(`{${name}`);
                                return (
                                    <span key={name} className={cn("inline-flex h-5 items-center gap-1 rounded-sm border px-1.5 font-mono text-[11px] leading-none", present ? "border-success/32 bg-success/8 text-success-foreground" : "border-border bg-muted text-muted-foreground")}>
                                        <span className="size-1.25 rounded-full" style={{ background: present ? "var(--success-foreground)" : "var(--muted-foreground)" }} />
                                        {`{${name}}`}
                                    </span>
                                );
                            })}
                        </div>
                    )}

                    <div className="mt-3.5">
                        <MonoSection>{t("i18n.sourceHash")}</MonoSection>
                    </div>
                    <p className="mt-1.5 break-all font-mono text-[11px] text-muted-foreground">{entry.source_hash}</p>
                </div>
            </div>
        </Card>
    );
}

// ---------------------------------------------------------------- history drawer

function HistoryDrawer({ locale, messageKey, canWrite, authed, onClose }: { locale: string; messageKey: string; canWrite: boolean; authed: boolean; onClose: () => void }): React.ReactElement {
    const t: TranslationsT = useT("admin");
    const rt: TranslationsRichT = useRichT("admin");
    const fmt = useFormatters();
    const queryClient = useQueryClient();
    const auditQuery = useQuery(translationEntryAuditQueryOptions({ locale, key: messageKey }, authed));

    const revert = useMutation({
        mutationFn: (input: IUpdateTranslationInput) => updateTranslationFn({ data: input }),
        onSuccess: (result) => {
            if (!result.ok) {
                toastManager.add({ id: `i18n-revert-invalid-${Date.now()}`, title: t("i18n.toast.revertRejected"), description: result.details.map((d) => d.message).join(" · ") || result.message, type: "error" });
                return;
            }
            void queryClient.invalidateQueries({ queryKey: ["admin", "i18n", "messages"] });
            void queryClient.invalidateQueries({ queryKey: ["admin", "i18n", "progress"] });
            void queryClient.invalidateQueries({ queryKey: ["admin", "i18n", "audit"] });
            toastManager.add({ id: `i18n-revert-${Date.now()}`, title: t("i18n.toast.reverted"), description: t("i18n.toast.reverted.desc", { key: messageKey }), type: "success" });
        },
        onError: (err: unknown) => toastManager.add({ id: `i18n-revert-err-${Date.now()}`, title: t("i18n.toast.revertFailed"), description: err instanceof Error ? err.message : String(err), type: "error" }),
    });

    const revisions = auditQuery.data ?? [];

    return (
        <>
            <button type="button" className="fixed inset-0 z-50 cursor-default bg-black/36 backdrop-blur-[2px]" onClick={onClose} aria-label={t("i18n.closeDrawer")} />
            <aside className="fixed top-0 right-0 bottom-0 z-51 flex w-120 max-w-[92vw] flex-col border-border border-l bg-background shadow-[-20px_0_60px_oklch(0_0_0/0.18)]">
                <div className="flex items-start justify-between gap-2 border-border border-b px-4.5 py-3.5">
                    <div className="flex min-w-0 flex-col gap-0.5">
                        <span className="font-bold text-[10px] text-primary uppercase tracking-[0.22em]">{t("i18n.history.kicker", { locale })}</span>
                        <span className="truncate font-mono text-[12.5px]">{messageKey}</span>
                        <span className="text-[11.5px] text-muted-foreground">{t("i18n.history.count", { count: revisions.length })}</span>
                    </div>
                    <Button variant="ghost" size="icon-sm" onClick={onClose} aria-label={t("i18n.close")}>
                        <XIcon />
                    </Button>
                </div>
                <div className="flex-1 overflow-auto p-4.5">
                    {auditQuery.isPending ? (
                        <Skeleton className="h-40 w-full" />
                    ) : revisions.length === 0 ? (
                        <div className="py-10 text-center text-[13px] text-muted-foreground">{t("i18n.history.empty")}</div>
                    ) : (
                        <div className="relative pl-4">
                            <div className="absolute top-1 bottom-1 left-1 w-0.5 rounded bg-border" />
                            {revisions.map((rev, i) => (
                                <div key={rev.id} className="relative pb-4 last:pb-0">
                                    <span className={cn("absolute top-1 -left-4.5 size-2.5 rounded-full border-2 bg-card shadow-[0_0_0_3px_var(--background)]", i === 0 ? "border-primary" : "border-border")} />
                                    <div className="font-medium font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.06em]">{fmt.relativeShort(rev.changed_at)}</div>
                                    <div className="mt-1 text-[12.5px] leading-normal">{rt("i18n.history.changedKey", { actor: <span className="font-mono text-muted-foreground">{rev.changed_by.slice(0, 8)}…</span> })}</div>
                                    <div className="mt-1.5 rounded-lg border border-border bg-[color-mix(in_srgb,var(--card),oklch(0_0_0)_2%)] p-2 font-mono text-[11.5px] leading-normal">
                                        {rev.old_value ? <div className="rounded-[3px] bg-destructive/8 px-1 text-destructive-foreground line-through decoration-destructive/40">{rev.old_value}</div> : <div className="px-1 text-muted-foreground italic">{t("i18n.history.wasUntranslated")}</div>}
                                        {rev.new_value ? <div className="rounded-[3px] bg-emerald-500/8 px-1 text-emerald-700 dark:text-emerald-300">{rev.new_value}</div> : <div className="px-1 text-muted-foreground italic">{t("i18n.history.cleared")}</div>}
                                    </div>
                                    {rev.old_value !== null ? (
                                        <div className="mt-1.5">
                                            <Button variant="outline" size="xs" disabled={!canWrite || revert.isPending} onClick={() => revert.mutate({ locale, key: messageKey, value: rev.old_value ?? "" })}>
                                                <RotateCcwIcon />
                                                {t("i18n.history.revert")}
                                            </Button>
                                        </div>
                                    ) : null}
                                </div>
                            ))}
                        </div>
                    )}
                </div>
            </aside>
        </>
    );
}

// ---------------------------------------------------------------- grants

function GrantsSection({ locale, userId, role, authed, onGrant }: { locale: string; userId: string | null; role: string | null; authed: boolean; onGrant: () => void }): React.ReactElement | null {
    const t: TranslationsT = useT("admin");
    const rt: TranslationsRichT = useRichT("admin");
    const fmt = useFormatters();
    const queryClient = useQueryClient();
    const permsQuery = useQuery(translationPermissionsQueryOptions(locale || undefined, authed));
    const perms = permsQuery.data ?? [];

    // The backend needs `admin` on the locale to hand out grants, and
    // `writable-locales` only reports edit-level access - so the own-row lookup
    // is what tells the UI whether the manage controls are usable.
    const canManage = isSuperAdmin(role) || (userId !== null && perms.some((p) => p.user_id === userId && p.permission === "admin"));

    const revoke = useMutation({
        mutationFn: (input: IGrantTranslationPermissionInput) => revokeTranslationPermissionFn({ data: input }),
        onSuccess: () => {
            void queryClient.invalidateQueries({ queryKey: ["admin", "i18n", "permissions"] });
            void queryClient.invalidateQueries({ queryKey: ["admin", "i18n", "writable-locales"] });
            toastManager.add({ id: `i18n-perm-revoke-${Date.now()}`, title: t("i18n.toast.grantRevoked"), description: t("i18n.toast.grantRevoked.desc", { locale }), type: "success" });
        },
        onError: (err: unknown) => toastManager.add({ id: `i18n-perm-revoke-err-${Date.now()}`, title: t("i18n.toast.revokeFailed"), description: err instanceof Error ? err.message : String(err), type: "error" }),
    });

    if (!canManage) return null;

    return (
        <Card>
            <CardHeader>
                <CardTitle className="text-sm">
                    {t("i18n.grants.title")} <span className="font-mono">{locale || "-"}</span>
                </CardTitle>
                <CardDescription className="text-xs">{rt("i18n.grants.desc", { table: <HCode>translation_permissions</HCode> })}</CardDescription>
                <CardAction>
                    <Button size="sm" disabled={!locale} onClick={onGrant}>
                        <PlusIcon />
                        {t("i18n.grants.add")}
                    </Button>
                </CardAction>
            </CardHeader>
            {permsQuery.isPending ? (
                <CardContent className="border-border border-t">
                    <Skeleton className="h-24 w-full" />
                </CardContent>
            ) : perms.length === 0 ? (
                <CardContent className="border-border border-t py-10 text-center text-[13px] text-muted-foreground">{t("i18n.grants.empty")}</CardContent>
            ) : (
                <div className="overflow-x-auto border-border border-t">
                    <table className="w-full min-w-150 border-collapse text-[13px]">
                        <thead>
                            <tr>
                                {[t("i18n.th.user"), t("i18n.th.level"), t("i18n.th.granted"), t("i18n.th.grantedBy"), ""].map((h) => (
                                    <th key={h} className="bg-[color-mix(in_srgb,var(--card),oklch(0_0_0)_1.5%)] px-3.5 py-2.5 text-left font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em]">
                                        {h}
                                    </th>
                                ))}
                            </tr>
                        </thead>
                        <tbody>
                            {perms.map((p) => (
                                <tr key={`${p.user_id}-${p.permission}`} className="border-border border-b last:border-0">
                                    <td className="px-3.5 py-2.5 font-mono text-[11.5px] text-muted-foreground">{p.user_id.slice(0, 8)}…</td>
                                    <td className="px-3.5 py-2.5">
                                        <Badge variant={p.permission === "admin" ? "default" : "outline"}>{levelLabel(t, p.permission)}</Badge>
                                    </td>
                                    <td className="px-3.5 py-2.5 text-muted-foreground">{fmt.date(p.granted_at, { year: "numeric", month: "numeric", day: "numeric" })}</td>
                                    <td className="px-3.5 py-2.5 font-mono text-[11.5px] text-muted-foreground">{p.granted_by ? `${p.granted_by.slice(0, 8)}…` : "-"}</td>
                                    <td className="px-3.5 py-2.5">
                                        <Button variant="destructive-outline" size="xs" disabled={revoke.isPending} onClick={() => revoke.mutate({ locale: p.locale, userId: p.user_id, permission: p.permission as TierListPermissionLevel })}>
                                            <Trash2Icon />
                                            {t("i18n.revoke")}
                                        </Button>
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            )}
        </Card>
    );
}

function GrantDialog({ locale, onClose }: { locale: string; onClose: () => void }): React.ReactElement {
    const t: TranslationsT = useT("admin");
    const queryClient = useQueryClient();
    const [q, setQ] = useState("");
    const [selectedUserId, setSelectedUserId] = useState<string | null>(null);
    const [permission, setPermission] = useState<TierListPermissionLevel>("edit");
    // One trimmed character is a real query: the backend trims and ILIKEs anything non-empty,
    // and a CJK nickname prefix is a single code unit. Debounce instead of gating on length.
    const term = useDebounce(q.trim(), 350);
    const searchQuery = useQuery({ ...searchUsersQueryOptions({ q: term, limit: 8 }), enabled: term.length > 0 });

    const grant = useMutation({
        mutationFn: (input: IGrantTranslationPermissionInput) => grantTranslationPermissionFn({ data: input }),
        onSuccess: () => {
            void queryClient.invalidateQueries({ queryKey: ["admin", "i18n", "permissions"] });
            void queryClient.invalidateQueries({ queryKey: ["admin", "i18n", "writable-locales"] });
            toastManager.add({ id: `i18n-perm-create-${Date.now()}`, title: t("i18n.toast.grantCreated"), description: t("i18n.toast.grantCreated.desc", { level: levelLabel(t, permission), locale }), type: "success" });
            onClose();
        },
        onError: (err: unknown) => toastManager.add({ id: `i18n-perm-create-err-${Date.now()}`, title: t("i18n.toast.grantFailed"), description: err instanceof Error ? err.message : String(err), type: "error" }),
    });

    const results = searchQuery.data?.entries ?? [];

    return (
        <>
            <button type="button" className="fixed inset-0 z-55 cursor-default bg-black/36 backdrop-blur-[2px]" onClick={onClose} aria-label={t("i18n.close")} />
            <div className="pointer-events-none fixed inset-0 z-60 grid place-items-center p-3 max-sm:items-end max-sm:p-0">
                <div className="pointer-events-auto flex max-h-[92dvh] w-120 max-w-[92vw] flex-col overflow-hidden rounded-2xl border border-border bg-background shadow-[0_30px_60px_oklch(0_0_0/0.35),0_8px_18px_oklch(0_0_0/0.2)] max-sm:w-full max-sm:max-w-none max-sm:rounded-b-none max-sm:border-x-0 max-sm:border-b-0">
                    <div className="shrink-0 border-border border-b px-5 pt-4 pb-3.5">
                        <span className="font-bold text-[10px] text-primary uppercase tracking-[0.22em]">{t("i18n.grant.kicker")}</span>
                        <div className="mt-1.5 font-semibold text-[18px] leading-tight tracking-[-0.01em]">{t("i18n.grant.title", { locale })}</div>
                    </div>
                    <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto p-5 [-webkit-overflow-scrolling:touch]">
                        <div className="flex flex-col gap-1.5">
                            <span className="font-medium text-[12px]">{t("i18n.grant.findDoctor")}</span>
                            <InputGroup>
                                <InputGroupAddon>
                                    <SearchIcon />
                                </InputGroupAddon>
                                <Input
                                    placeholder={t("i18n.grant.searchPlaceholder")}
                                    size="sm"
                                    value={q}
                                    onChange={(e) => {
                                        setQ(e.target.value);
                                        setSelectedUserId(null);
                                    }}
                                />
                            </InputGroup>
                            {q.trim().length > 0 ? (
                                term !== q.trim() || searchQuery.isPending ? (
                                    <Skeleton className="h-12 w-full" />
                                ) : results.length === 0 ? (
                                    <div className="text-[12.5px] text-muted-foreground">{t("i18n.grant.noMatches")}</div>
                                ) : (
                                    <ul className="max-h-48 overflow-auto rounded-lg border border-border">
                                        {results.map((r) => (
                                            <li key={r.id}>
                                                <button
                                                    type="button"
                                                    onClick={() => setSelectedUserId(r.id)}
                                                    className={cn("flex w-full items-center justify-between gap-2 border-0 border-border border-b px-3 py-2 text-left last:border-0 hover:bg-accent", selectedUserId === r.id && "bg-[color-mix(in_srgb,var(--primary)_6%,var(--card))]")}
                                                >
                                                    <span className="flex items-center gap-2">
                                                        <span className="relative inline-block size-6 overflow-hidden rounded-full bg-[linear-gradient(135deg,oklch(0.58_0.22_25),oklch(0.85_0.12_25))]">
                                                            <img src={getSecretaryAvatarURL({ secretary: r.secretary, secretary_skin_id: r.secretary_skin_id })} alt="" loading="lazy" className="absolute inset-0 size-full object-cover" onError={(e) => (e.currentTarget.style.display = "none")} />
                                                        </span>
                                                        <span>
                                                            <span className="font-medium">{r.nickname ?? "-"}</span>
                                                            <span className="ml-1.5 font-mono text-[11.5px] text-muted-foreground">{t("i18n.uid", { uid: r.uid })}</span>
                                                        </span>
                                                    </span>
                                                    <span className="font-mono text-[11px] text-muted-foreground">{r.role}</span>
                                                </button>
                                            </li>
                                        ))}
                                    </ul>
                                )
                            ) : null}
                        </div>
                        <div className="flex flex-col gap-1.5">
                            <span className="font-medium text-[12px]">{t("i18n.grant.level")}</span>
                            <div className="inline-flex gap-px self-start rounded-lg border border-border bg-card p-0.5">
                                {GRANT_LEVELS.map((l) => (
                                    <button
                                        type="button"
                                        key={l}
                                        onClick={() => setPermission(l)}
                                        className={cn(
                                            "cursor-pointer rounded-md px-2 py-1 font-medium text-[11.5px] transition-colors",
                                            l === permission && l === "admin" && "bg-primary text-primary-foreground",
                                            l === permission && l !== "admin" && "bg-[color-mix(in_srgb,var(--foreground)_6%,transparent)] text-foreground",
                                            l !== permission && "text-muted-foreground hover:text-foreground",
                                        )}
                                    >
                                        {levelLabel(t, l)}
                                    </button>
                                ))}
                            </div>
                            <span className="text-[11px] text-muted-foreground">{t("i18n.grant.levelHint")}</span>
                        </div>
                    </div>
                    <div className="flex shrink-0 justify-end gap-2 border-border border-t p-3.5">
                        <Button variant="outline" size="sm" onClick={onClose}>
                            {t("i18n.cancel")}
                        </Button>
                        <Button size="sm" disabled={!selectedUserId || grant.isPending} loading={grant.isPending} onClick={() => selectedUserId && grant.mutate({ locale, userId: selectedUserId, permission })}>
                            {t("i18n.grant.submit", { level: levelLabel(t, permission) })}
                        </Button>
                    </div>
                </div>
            </div>
        </>
    );
}
