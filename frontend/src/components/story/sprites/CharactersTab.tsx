import { useNavigate, useSearch } from "@tanstack/react-router";
import type React from "react";
import { useCallback, useDeferredValue, useMemo, useState } from "react";
import { SearchField } from "#/components/SearchField";
import { pageOf, RECORD_PAGE } from "#/components/story/library/impl/paging";
import { Button } from "#/components/ui/button";
import { Skeleton } from "#/components/ui/skeleton";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { useFormatters, useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./CharactersTab.messages";
import { useSpriteIndex } from "./data";
import { filterSprites, prepareSpriteSearch, primaryName, SPRITE_KIND_FILTERS, SPRITE_SORTS, type SpriteKindFilter, type SpriteSort } from "./gallery";
import { SpriteCard } from "./SpriteCard";
import { SpriteSheetDialog } from "./SpriteSheet";

type SpritesT = TypedT<typeof messages>;

const GRID = "grid grid-cols-2 gap-2.5 min-[480px]:grid-cols-3 sm:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6";
/** Placeholder cards while the list loads: two rows at the widest grid. */
const SKELETON_CARDS = 12;

/**
 * The library's CHARACTERS tab: every story sprite folder, searchable by the
 * names the scripts speak it under. It is loaded lazily by `StoryLibrary`, so
 * Browse's bundle does not carry it, and its list query runs only once the
 * tab has been opened.
 *
 * The library's performance rules apply: the controls answer on the immediate
 * state and the grid reads DEFERRED copies (`useDeferredValue`), the
 * haystacks are prepared once per list, cards are
 * memoised with stable keys and a stable `onOpen`, the grid mounts a page of
 * {@link RECORD_PAGE} at a time, the thumbs are lazy and nothing samples a
 * palette. The open sheet is `?sprite=<base>` and is read HERE, not in
 * `StoryLibrary`, so opening one re-renders this tab and not the library.
 */
export default function CharactersTab(): React.ReactElement {
    const t: SpritesT = useT("story");
    const f = useFormatters();
    const locale = useLocale();
    const q = useSpriteIndex();
    const entries = q.data?.sprites;

    const [query, setQuery] = useState("");
    const [kind, setKind] = useState<SpriteKindFilter>("all");
    const [sort, setSort] = useState<SpriteSort>("appearances");
    const deferredQuery = useDeferredValue(query);
    const deferredKind = useDeferredValue(kind);
    const deferredSort = useDeferredValue(sort);

    const search = useMemo(() => prepareSpriteSearch(entries ?? []), [entries]);
    const collator = useMemo(() => new Intl.Collator(locale, { sensitivity: "base", numeric: true }), [locale]);
    const visible = useMemo(() => filterSprites(search, deferredQuery, deferredKind, deferredSort, collator), [search, deferredQuery, deferredKind, deferredSort, collator]);

    // The page limit is remembered against the controls it was raised under,
    // so a new search starts from one page without an effect.
    const pageKey = `${deferredQuery}\u0000${deferredKind}\u0000${deferredSort}`;
    const [raised, setRaised] = useState<{ key: string; limit: number }>({ key: pageKey, limit: RECORD_PAGE });
    const limit = raised.key === pageKey ? raised.limit : RECORD_PAGE;
    const { shown, hidden } = pageOf(visible, limit);
    const more = Math.min(RECORD_PAGE, hidden);

    const open = useSearch({ from: "/stories", select: (search) => search.sprite ?? null });
    const navigate = useNavigate({ from: "/stories" });
    const onOpen = useCallback((base: string) => void navigate({ search: (prev) => ({ ...prev, tab: "characters", sprite: base }), replace: true, resetScroll: false }), [navigate]);
    const onClose = useCallback(() => void navigate({ search: (prev) => ({ ...prev, sprite: undefined }), replace: true, resetScroll: false }), [navigate]);
    const openEntry = useMemo(() => (open && entries ? entries.find((e) => e.base === open) : undefined), [open, entries]);

    const counts = useMemo(() => {
        const all = entries ?? [];
        const operators = all.filter((e) => e.kind === "operator").length;
        return { all: all.length, operators, npcs: all.length - operators };
    }, [entries]);

    return (
        <div className="pt-4">
            {entries ? <p className="mt-0 mb-3 max-w-2xl font-sans text-[12.5px] text-muted-foreground">{t("sprites.subtitle", { count: f.number(counts.all), operators: f.number(counts.operators), npcs: f.number(counts.npcs) })}</p> : null}
            <div className="mb-4 flex flex-wrap items-center gap-x-3 gap-y-2.5">
                <SearchField className="h-11 min-w-0 flex-1 basis-full sm:h-9.5 sm:max-w-100 sm:basis-auto" value={query} onChange={setQuery} placeholder={t("sprites.search.placeholder")} label={t("sprites.search.aria")} />
                <Segmented label={t("sprites.kind.aria")} options={SPRITE_KIND_FILTERS} value={kind} onChange={setKind} render={(k) => t(`sprites.kind.${k}`)} />
                <Segmented label={t("sprites.sort.aria")} options={SPRITE_SORTS} value={sort} onChange={setSort} render={(s) => t(`sprites.sort.${s}`)} />
                {entries ? <span className="font-mono text-[11px] text-muted-foreground tabular-nums">{t("sprites.results", { count: visible.length })}</span> : null}
            </div>

            <GalleryBody loading={q.isLoading} failed={q.isError} unavailable={q.isSuccess && q.data === null} empty={entries !== undefined && visible.length === 0}>
                <div className={GRID}>
                    {shown.map((entry) => (
                        <SpriteCard key={entry.base} entry={entry} onOpen={onOpen} />
                    ))}
                </div>
                {hidden > 0 ? (
                    <div className="mt-4 flex justify-center">
                        <Button variant="outline" className="max-sm:min-h-11" onClick={() => setRaised({ key: pageKey, limit: limit + more })}>
                            {t("sprites.showMore", { count: more, remaining: hidden })}
                        </Button>
                    </div>
                ) : null}
            </GalleryBody>

            <SpriteSheetDialog base={open} name={openEntry ? primaryName(openEntry) : (open ?? "")} onClose={onClose} />
        </div>
    );
}

/** A row of mutually exclusive buttons, the same pill strip the illustration panel uses. */
function Segmented<T extends string>({ label, options, value, onChange, render }: { label: string; options: readonly T[]; value: T; onChange: (v: T) => void; render: (v: T) => string }): React.ReactElement {
    return (
        <ToggleGroup aria-label={label} value={[value]} onValueChange={(next) => next[0] && onChange(next[0] as (typeof options)[number])} variant="outline" className="msv-scroll max-w-full shrink-0 overflow-x-auto">
            {options.map((o) => (
                <ToggleGroupItem key={o} value={o} className="px-2.5 font-semibold text-[11.5px] sm:text-[11.5px]">
                    {render(o)}
                </ToggleGroupItem>
            ))}
        </ToggleGroup>
    );
}

/** The grid's place: the unavailable, failed, loading and empty states, else the grid itself. */
function GalleryBody({ loading, failed, unavailable, empty, children }: { loading: boolean; failed: boolean; unavailable: boolean; empty: boolean; children: React.ReactNode }): React.ReactElement {
    const t: SpritesT = useT("story");
    if (unavailable) {
        return (
            <div className="rounded-2xl border border-border border-dashed p-10 text-center">
                <p className="m-0 font-sans text-[13px] text-muted-foreground">{t("sprites.unavailable")}</p>
                <p className="mt-1.5 mb-0 font-mono text-[11px] text-subtle-foreground">{t("sprites.unavailableNote")}</p>
            </div>
        );
    }
    if (failed) return <p className="py-10 text-center font-sans text-[13px] text-muted-foreground">{t("sprites.failed")}</p>;
    if (loading) {
        return (
            <output className={GRID} aria-label={t("sprites.loading")}>
                {Array.from({ length: SKELETON_CARDS }, (_, i) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: a placeholder has no identity beyond its position
                    <Skeleton key={i} className="aspect-3/4 w-full rounded-2xl" />
                ))}
            </output>
        );
    }
    if (empty) return <div className="rounded-2xl border border-border border-dashed p-14 text-center font-sans text-[14px] text-muted-foreground">{t("sprites.empty")}</div>;
    return <>{children}</>;
}
