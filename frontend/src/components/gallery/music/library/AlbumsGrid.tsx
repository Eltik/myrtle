import { DiscIcon } from "lucide-react";
import type React from "react";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { sizedCover } from "../impl/cover";
import type { IAlbum } from "../impl/model";
import { useMusicNav } from "../impl/nav";
import type { messages } from "../MusicGallery.messages";
import { usePlayer } from "../player/engine";
import { NOTICE_CLASS, ShowMore, usePagedList } from "./paging";

type MusicT = TypedT<typeof messages>;

/**
 * THE ALBUMS GRID: square covers with the name over a bottom scrim. A page of
 * covers mounts at a time behind "Show more"; a click opens the album.
 *
 * Hover is one motion, the cover easing up to 104%, plus two fades (the dim
 * lifting, the ring showing): transform and opacity only. The card used to
 * lift, zoom, re-filter and slide its label at once, and transitioned
 * `filter` and `box-shadow`, which repaint every frame.
 */
export function AlbumsGrid({ albums, pageKey }: { albums: readonly IAlbum[]; pageKey: string }): React.ReactElement {
    const t: MusicT = useT("gallery");
    const nav = useMusicNav();
    const playingAlbum = usePlayer((s) => s.current?.albumId ?? null);
    const page = usePagedList(albums, pageKey);

    if (albums.length === 0) return <p className={NOTICE_CLASS}>{t("music.list.noAlbums")}</p>;

    return (
        <div>
            <ul className="m-0 grid list-none grid-cols-[repeat(auto-fill,minmax(150px,1fr))] gap-3.5 p-0 sm:grid-cols-[repeat(auto-fill,minmax(170px,1fr))]">
                {page.shown.map((album) => (
                    <li key={album.id}>
                        <button
                            type="button"
                            onClick={() => nav.openAlbum(album.id)}
                            className={cn("group/card relative block aspect-square w-full cursor-pointer overflow-hidden rounded-lg bg-muted text-left outline-none focus-visible:ring-2 focus-visible:ring-ring/60", album.id === playingAlbum && "ring-2 ring-primary")}
                        >
                            <DiscIcon aria-hidden="true" className="absolute inset-0 m-auto size-1/3 text-muted-foreground/40" />
                            <img referrerPolicy="no-referrer" src={sizedCover(album.coverUrl, 240)} alt="" loading="lazy" decoding="async" className="relative size-full object-cover transition-transform duration-200 ease-out group-hover/card:scale-104 motion-reduce:transition-none" />
                            <span aria-hidden="true" className="pointer-events-none absolute inset-0 bg-black/18 transition-opacity duration-200 ease-out group-hover/card:opacity-30" />
                            <span className="pointer-events-none absolute inset-0 bg-[linear-gradient(to_top,rgb(0_0_0/.88),rgb(0_0_0/.15)_55%,transparent_75%)]" />
                            <span className="absolute inset-x-0 bottom-0 flex flex-col gap-0.5 p-2.5">
                                <span className="line-clamp-2 font-sans font-semibold text-[12.5px] text-white leading-tight [text-shadow:0_1px_2px_rgb(0_0_0/.6)]">{album.name}</span>
                                <span className="font-mono text-[9.5px] text-white/70 uppercase tracking-[0.12em]">{t("music.list.trackCount", { count: album.trackCount })}</span>
                            </span>
                            <span aria-hidden="true" className="pointer-events-none absolute inset-0 rounded-lg opacity-0 ring-2 ring-primary/60 ring-inset transition-opacity duration-150 ease-out group-hover/card:opacity-100" />
                        </button>
                    </li>
                ))}
            </ul>
            <ShowMore page={page} />
        </div>
    );
}
