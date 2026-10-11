import { ChartColumnIcon, DiscIcon, Gamepad2Icon, HeartIcon, HistoryIcon, LibraryIcon, ListMusicIcon, MicVocalIcon, PlusIcon } from "lucide-react";
import type React from "react";
import { useState } from "react";
import { Button } from "#/components/ui/button";
import { Select, SelectContent, SelectGroup, SelectGroupLabel, SelectItem, SelectSeparator, SelectTrigger, SelectValue } from "#/components/ui/select";
import { toastManager } from "#/components/ui/toast";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { createPlaylist, useLibrary } from "../impl/library/store";
import { useMusicNav } from "../impl/nav";
import { LIBRARY_VIEWS, type LibrarySelection, type LibraryView } from "../impl/route";
import type { messages } from "../MusicGallery.messages";
import { PlaylistNameDialog } from "./PlaylistNameDialog";

type MusicT = TypedT<typeof messages>;

const ICONS: Record<LibraryView, React.ComponentType<{ className?: string; "aria-hidden"?: boolean }>> = {
    albums: DiscIcon,
    songs: LibraryIcon,
    artists: MicVocalIcon,
    liked: HeartIcon,
    history: HistoryIcon,
    stats: ChartColumnIcon,
    game: Gamepad2Icon,
};

function keyOf(selection: LibrarySelection): string {
    return selection.kind === "view" ? `view:${selection.view}` : selection.kind === "playlist" ? `playlist:${selection.id}` : `artist:${selection.name}`;
}

/**
 * THE LIBRARY RAIL: the fixed views, then the playlists with a `+`. On phones
 * the same list is a select, with the `+` beside it.
 */
export function Rail({ selection }: { selection: LibrarySelection }): React.ReactElement {
    const t: MusicT = useT("gallery");
    const nav = useMusicNav();
    const playlists = useLibrary((s) => s.playlists);
    const playlistLimit = useLibrary((s) => s.playlistLimit);
    const [naming, setNaming] = useState(false);
    const active = keyOf(selection);

    const create = (name: string) => {
        void createPlaylist(name).then((pl) => {
            if (pl) nav.openLibrary({ kind: "playlist", id: pl.id });
            else toastManager.add({ title: t("music.playlist.createFailed"), type: "error" });
        });
    };
    const newButton = (
        <Button size="icon-sm" variant="ghost" className="max-sm:size-11" disabled={playlists.length >= playlistLimit} onClick={() => setNaming(true)} aria-label={t("music.playlist.new")} title={t("music.playlist.new")}>
            <PlusIcon aria-hidden="true" />
        </Button>
    );
    const label = (v: LibraryView) => t(`music.rail.${v}`);
    const current = selection.kind === "view" ? label(selection.view) : selection.kind === "artist" ? t("music.rail.artistView", { name: selection.name }) : (playlists.find((p) => p.id === selection.id)?.name ?? t("music.rail.playlists"));

    return (
        <>
            {/* Phones: a select over the same list. */}
            <div className="flex items-center gap-2 md:hidden">
                <Select
                    value={active}
                    onValueChange={(next: string | null) => {
                        if (!next) return;
                        const [kind, ...rest] = next.split(":");
                        const value = rest.join(":");
                        if (kind === "playlist") nav.openLibrary({ kind: "playlist", id: value });
                        else if ((LIBRARY_VIEWS as readonly string[]).includes(value)) nav.openLibrary({ kind: "view", view: value as LibraryView });
                    }}
                >
                    <SelectTrigger className="min-h-11 flex-1" aria-label={t("music.rail.aria")}>
                        <SelectValue>{() => current}</SelectValue>
                    </SelectTrigger>
                    <SelectContent>
                        {LIBRARY_VIEWS.map((v) => (
                            <SelectItem key={v} value={`view:${v}`}>
                                {label(v)}
                            </SelectItem>
                        ))}
                        {selection.kind === "artist" ? <SelectItem value={active}>{current}</SelectItem> : null}
                        {playlists.length > 0 ? (
                            <>
                                <SelectSeparator />
                                <SelectGroup>
                                    <SelectGroupLabel>{t("music.rail.playlists")}</SelectGroupLabel>
                                    {playlists.map((p) => (
                                        <SelectItem key={p.id} value={`playlist:${p.id}`}>
                                            {p.name}
                                        </SelectItem>
                                    ))}
                                </SelectGroup>
                            </>
                        ) : null}
                    </SelectContent>
                </Select>
                {newButton}
            </div>

            <nav aria-label={t("music.rail.aria")} className="hidden self-start rounded-xl border border-border bg-card/55 p-2 md:sticky md:top-20 md:block">
                <ul className="m-0 flex list-none flex-col gap-0.5 p-0">
                    {LIBRARY_VIEWS.map((v) => {
                        const Icon = ICONS[v];
                        const on = active === `view:${v}`;
                        return (
                            <li key={v}>
                                <RailButton on={on} onClick={() => nav.openLibrary({ kind: "view", view: v })}>
                                    <Icon className="size-4 shrink-0" aria-hidden />
                                    <span className="min-w-0 flex-1 truncate">{label(v)}</span>
                                </RailButton>
                            </li>
                        );
                    })}
                </ul>
                <div className="mt-3 flex items-center justify-between border-border border-t px-1.5 pt-2.5">
                    <span className="flex items-center gap-1.5 font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">
                        <ListMusicIcon className="size-3.5" aria-hidden="true" />
                        {t("music.rail.playlists")}
                    </span>
                    {newButton}
                </div>
                {playlists.length === 0 ? (
                    <p className="m-0 px-2 py-2 font-sans text-[12px] text-muted-foreground">{t("music.rail.noPlaylists")}</p>
                ) : (
                    <ul className="m-0 mt-1 flex list-none flex-col gap-0.5 p-0">
                        {playlists.map((p) => (
                            <li key={p.id}>
                                <RailButton on={active === `playlist:${p.id}`} onClick={() => nav.openLibrary({ kind: "playlist", id: p.id })}>
                                    <span className="min-w-0 flex-1 truncate">{p.name}</span>
                                    <span className="shrink-0 rounded-sm bg-secondary in-aria-[current=page]:bg-primary-foreground/20 px-1.5 font-mono in-aria-[current=page]:text-primary-foreground text-[10.5px] text-muted-foreground tabular-nums">{p.trackIds.length}</span>
                                </RailButton>
                            </li>
                        ))}
                    </ul>
                )}
            </nav>
            <PlaylistNameDialog open={naming} onOpenChange={setNaming} mode="create" onSubmit={create} />
        </>
    );
}

function RailButton({ on, onClick, children }: { on: boolean; onClick: () => void; children: React.ReactNode }): React.ReactElement {
    return (
        <button
            type="button"
            onClick={onClick}
            aria-current={on ? "page" : undefined}
            className={cn("flex min-h-9 pointer-coarse:min-h-11 w-full cursor-pointer items-center gap-2.5 rounded-md px-2.5 text-left font-sans text-[13px] transition-colors", on ? "bg-primary font-semibold text-primary-foreground" : "text-foreground hover:bg-secondary/70")}
        >
            {children}
        </button>
    );
}
