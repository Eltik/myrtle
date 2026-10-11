import type React from "react";
import { useMemo, useState } from "react";
import { Command, CommandDialog, CommandDialogPopup, CommandEmpty, CommandInput, CommandItem, CommandList, CommandPanel } from "#/components/ui/command";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { searchAndRank } from "#/lib/search/fuzzy";
import { sizedCover } from "../impl/cover";
import type { IAlbum } from "../impl/model";
import type { messages } from "../MusicGallery.messages";

type MusicT = TypedT<typeof messages>;

/** Albums the palette lists at once. */
const MAX_RESULTS = 40;

/**
 * "Jump to album": a command palette over the release names (and their
 * credited artists), best match first. Picking one moves the coverflow to it;
 * it does not open it.
 */
export function JumpPalette({ open, onOpenChange, albums, onPick }: { open: boolean; onOpenChange: (open: boolean) => void; albums: readonly IAlbum[]; onPick: (index: number) => void }): React.ReactElement {
    const t: MusicT = useT("gallery");
    const [query, setQuery] = useState("");
    const indexOf = useMemo(() => new Map(albums.map((a, i) => [a.id, i])), [albums]);
    const results = useMemo(() => {
        if (query.trim() === "") return albums.slice(0, MAX_RESULTS);
        return searchAndRank(query, albums, (a) => ({ name: a.name, aliases: a.artists }), MAX_RESULTS).map((r) => r.item);
    }, [albums, query]);

    return (
        <CommandDialog
            open={open}
            onOpenChange={(next) => {
                onOpenChange(next);
                if (!next) setQuery("");
            }}
        >
            <CommandDialogPopup>
                <Command value={query} onValueChange={setQuery} mode="none">
                    <CommandInput placeholder={t("music.discovery.jump.placeholder")} aria-label={t("music.discovery.jump")} />
                    <CommandPanel>
                        <CommandList>
                            {results.length === 0 ? <CommandEmpty>{t("music.discovery.jump.empty")}</CommandEmpty> : null}
                            {results.map((album) => (
                                <CommandItem key={album.id} value={album.id} onClick={() => onPick(indexOf.get(album.id) ?? 0)} className="flex cursor-pointer flex-row items-center gap-2.5">
                                    <img referrerPolicy="no-referrer" src={sizedCover(album.coverUrl, 32)} alt="" loading="lazy" decoding="async" className="size-8 shrink-0 rounded-[4px] bg-muted object-cover" />
                                    <span className="min-w-0 flex-1 truncate">{album.name}</span>
                                    <span className="shrink-0 text-muted-foreground text-xs tabular-nums">{t("music.list.trackCount", { count: album.trackCount })}</span>
                                </CommandItem>
                            ))}
                        </CommandList>
                    </CommandPanel>
                </Command>
            </CommandDialogPopup>
        </CommandDialog>
    );
}
