import { ArrowDownWideNarrowIcon, ArrowUpNarrowWideIcon, ListFilterIcon, SearchIcon } from "lucide-react";
import type React from "react";
import { useState } from "react";
import { Button } from "#/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Switch } from "#/components/ui/switch";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { MUSIC_KIND_FILTERS, type MusicKindFilter } from "../impl/tracks";
import type { messages } from "../MusicGallery.messages";
import { ALBUM_SORTS, type AlbumSort, SONG_SORTS, type SongSort } from "./sort";

type MusicT = TypedT<typeof messages>;

/** What the strip offers: the album sorts, the song sorts and filters, a search only, or the game's kind chips. */
export type StripMode = "albums" | "songs" | "search" | "game";

export interface IFilterStripProps {
    mode: StripMode;
    query: string;
    onQuery: (q: string) => void;
    albumSort: AlbumSort;
    onAlbumSort: (s: AlbumSort) => void;
    songSort: SongSort;
    onSongSort: (s: SongSort) => void;
    reversed: boolean;
    onReversed: (r: boolean) => void;
    hideInstrumentals: boolean;
    onHideInstrumentals: (v: boolean) => void;
    kind: MusicKindFilter;
    onKind: (k: MusicKindFilter) => void;
    onReset: () => void;
}

/**
 * THE LIBRARY'S FILTER STRIP: sort direction and sort order on the left, the
 * search box and the Filters toggle on the right, and the filters row under
 * it when open. Filters apply as they change.
 */
export function FilterStrip(props: IFilterStripProps): React.ReactElement {
    const t: MusicT = useT("gallery");
    const { mode, query, onQuery, reversed, onReversed } = props;
    const [open, setOpen] = useState(false);
    const sortable = mode === "albums" || mode === "songs";

    return (
        <div className="mb-4 rounded-xl border border-border bg-card/55 p-2">
            <div className="flex flex-wrap items-center gap-2">
                {sortable ? (
                    <>
                        <Button size="icon" variant="outline" className="max-sm:size-11" onClick={() => onReversed(!reversed)} aria-pressed={reversed} aria-label={t("music.filter.reverse")} title={t("music.filter.reverse")}>
                            {reversed ? <ArrowUpNarrowWideIcon aria-hidden="true" /> : <ArrowDownWideNarrowIcon aria-hidden="true" />}
                        </Button>
                        {mode === "albums" ? (
                            <SortSelect value={props.albumSort} options={ALBUM_SORTS} onChange={props.onAlbumSort} render={(s) => t(`music.filter.sort.album.${s}`)} />
                        ) : (
                            <SortSelect value={props.songSort} options={SONG_SORTS} onChange={props.onSongSort} render={(s) => t(`music.filter.sort.song.${s}`)} />
                        )}
                    </>
                ) : null}
                {mode === "game" ? <KindChips value={props.kind} onChange={props.onKind} /> : null}
                <div className="relative min-w-0 flex-1 basis-full sm:basis-60">
                    <SearchIcon className="pointer-events-none absolute top-1/2 left-3 size-3.5 -translate-y-1/2 text-muted-foreground" aria-hidden="true" />
                    <input
                        type="search"
                        value={query}
                        onChange={(e) => onQuery(e.target.value)}
                        placeholder={mode === "game" ? t("music.search.placeholder") : t("music.filter.searchPlaceholder")}
                        aria-label={t("music.search.aria")}
                        className="h-11 w-full rounded-[9px] border border-border bg-secondary/50 pr-3 pl-8.5 font-sans text-[13px] text-foreground outline-none focus-visible:border-primary focus-visible:ring-2 focus-visible:ring-ring/40 sm:h-9"
                    />
                </div>
                {mode === "songs" ? (
                    <Button variant={open ? "default" : "outline"} className="max-sm:min-h-11" onClick={() => setOpen((o) => !o)} aria-expanded={open} title={t("music.filter.filters")}>
                        <ListFilterIcon aria-hidden="true" />
                        {t("music.filter.filters")}
                    </Button>
                ) : null}
            </div>
            {open && mode === "songs" ? (
                <div className="mt-2 flex flex-wrap items-center justify-between gap-2 border-border border-t px-1 pt-2">
                    {/* biome-ignore lint/a11y/noLabelWithoutControl: the Switch renders its own button inside the label */}
                    <label className="flex min-h-11 cursor-pointer items-center gap-2 font-sans text-[12.5px] text-foreground sm:min-h-8">
                        <Switch checked={props.hideInstrumentals} onCheckedChange={props.onHideInstrumentals} />
                        {t("music.filter.hideInstrumentals")}
                    </label>
                    <Button size="sm" variant="ghost" className="max-sm:min-h-11" onClick={props.onReset}>
                        {t("music.filter.reset")}
                    </Button>
                </div>
            ) : null}
        </div>
    );
}

function SortSelect<T extends string>({ value, options, onChange, render }: { value: T; options: readonly T[]; onChange: (v: T) => void; render: (v: T) => string }): React.ReactElement {
    const t: MusicT = useT("gallery");
    return (
        <Select
            value={value}
            onValueChange={(next: T | null) => {
                if (next !== null) onChange(next);
            }}
        >
            <SelectTrigger className="w-auto min-w-40 max-sm:min-h-11" aria-label={t("music.filter.sortBy")} title={t("music.filter.sortBy")}>
                <SelectValue>{() => render(value)}</SelectValue>
            </SelectTrigger>
            <SelectContent>
                {options.map((o) => (
                    <SelectItem key={o} value={o}>
                        {render(o)}
                    </SelectItem>
                ))}
            </SelectContent>
        </Select>
    );
}

/** The game's kind filter, the pill strip the old page had. */
function KindChips({ value, onChange }: { value: MusicKindFilter; onChange: (k: MusicKindFilter) => void }): React.ReactElement {
    const t: MusicT = useT("gallery");
    return (
        <fieldset className="msv-scroll flex max-w-full shrink-0 gap-0.5 overflow-x-auto rounded-[9px] border border-border bg-secondary/45 p-0.75" aria-label={t("music.kind.aria")}>
            {MUSIC_KIND_FILTERS.map((k) => (
                <button
                    key={k}
                    type="button"
                    onClick={() => onChange(k)}
                    aria-pressed={value === k}
                    className={cn("h-11 shrink-0 cursor-pointer rounded-md px-2.5 font-sans font-semibold text-[11.5px] transition-colors sm:h-7.5", value === k ? "bg-background text-foreground shadow-sm/5" : "text-muted-foreground hover:text-foreground")}
                >
                    {t(`music.kind.${k}`)}
                </button>
            ))}
        </fieldset>
    );
}
