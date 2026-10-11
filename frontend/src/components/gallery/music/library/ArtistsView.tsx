import type React from "react";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { IArtist } from "../impl/model";
import { useMusicNav } from "../impl/nav";
import type { messages } from "../MusicGallery.messages";

type MusicT = TypedT<typeof messages>;

/** Guest artists, one button each; the house credit is left out by the model. */
export function ArtistsView({ artists }: { artists: readonly IArtist[] }): React.ReactElement {
    const t: MusicT = useT("gallery");
    const nav = useMusicNav();
    if (artists.length === 0) return <p className="rounded-xl border border-border border-dashed p-10 text-center font-sans text-[13px] text-muted-foreground">{t("music.list.noArtists")}</p>;
    return (
        <ul className="m-0 list-none divide-y divide-border p-0">
            {artists.map((a) => (
                <li key={a.name}>
                    <button type="button" onClick={() => nav.openLibrary({ kind: "artist", name: a.name })} className="flex min-h-11 w-full cursor-pointer items-center justify-between gap-3 rounded-md px-2 text-left font-sans text-[13.5px] text-foreground hover:bg-primary/6">
                        <span className="min-w-0 truncate">{a.name}</span>
                        <span className="shrink-0 font-mono text-[11px] text-muted-foreground tabular-nums">{t("music.list.trackCount", { count: a.trackCount })}</span>
                    </button>
                </li>
            ))}
        </ul>
    );
}
