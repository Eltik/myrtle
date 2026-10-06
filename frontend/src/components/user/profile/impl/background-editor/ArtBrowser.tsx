import { BookImageIcon, ClapperboardIcon, MountainIcon, UserRoundIcon } from "lucide-react";
import { memo, useMemo, useState } from "react";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { useMediaQuery } from "#/hooks/use-media-query";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { ProfileBackgroundKind } from "#/types/generated/ProfileBackgroundKind";
import { GALLERY_KINDS, type GalleryKind, isGalleryKind } from "../background";
import { type IGalleryFilter, NO_GALLERY_FILTER } from "../gallery";
import type { messages } from "./ArtBrowser.messages";
import { COUNT, SEGMENT_ITEM, sourceTab } from "./chips";
import { RailLabel, RailRow } from "./rail";
import { CharacterArt } from "./sources/CharacterArt";
import { GalleryBar, GalleryResultsHeader, GalleryTiles, galleryFilterActions, useGalleryView } from "./sources/Gallery";
import { CategoryChips, StoryPanel } from "./sources/GalleryFilters";
import { useGalleryCatalog } from "./useGalleryCatalog";
import { TILE_SIZES, type TileSize, useTileSize } from "./useTileSize";

/** What the browser lists: the character art (outfits and operators), or one gallery source. */
type Source = "characters" | GalleryKind;

const SOURCES = ["characters", ...GALLERY_KINDS] as const satisfies readonly Source[];
const SOURCE_ICONS = { characters: UserRoundIcon, archive_pic: BookImageIcon, story_cg: ClapperboardIcon, story_scene: MountainIcon } as const satisfies Record<Source, unknown>;

/**
 * The height under the editor's top bar and its collapsed preview strip, which the rail
 * and the character picker fill. `--strip` is set by the editor to the strip's height,
 * whether it shows or not: these rows only stick once the preview has scrolled up under
 * the strip, which is when it shows, and a height that followed the strip would change the
 * scroll range as it shows and flip it off again at the scroller's end (48 flips in 30
 * scroll steps on Character art in Phone mode, measured 2026-10-06).
 */
const UNDER_STRIP = "h-[calc(100dvh-3.5rem-var(--strip,0px))]";
const STICKY_UNDER_STRIP = "sticky top-[var(--strip,0px)]";

/** The art the draft shows, by kind and id alone: its crop and zoom are no business of the browser's. */
export interface IPickedArt {
    kind: ProfileBackgroundKind;
    id: string;
}

interface IArtBrowserProps {
    /** The art the draft shows, `null` for none. Kept the same object while the kind and id are, so a drag in the preview does not re-render the browser. */
    selected: IPickedArt | null;
    /** Kept the same function for the editor's life, for the same reason. */
    onPick: (kind: ProfileBackgroundKind, id: string) => void;
}

/**
 * The editor's lower half: every art the background can be. From `md` up a rail on the
 * left, sticky under the preview strip, picks the source (Character art, then the
 * gallery's Archives, Story CGs and Scenes, each with its icon and a muted count) and,
 * for a gallery source, narrows it by category chips and by a searchable story panel; the
 * tiles flow beside it in the editor's one scroller, under a frosted sticky search row
 * with the tile-size control. Narrower, the sources are a tab row and the filters stack
 * above the tiles. It opens on the source of the art the draft shows.
 *
 * Memoized on `selected` and `onPick`: the draft changes on every pointer move of a drag,
 * and before this the browser, its 120 mounted tiles or the character picker re-rendered
 * on each one (10 renders per 10 moves, measured 2026-10-06).
 */
export const ArtBrowser = memo(function ArtBrowser({ selected, onPick }: IArtBrowserProps) {
    const t: TypedT<typeof messages> = useT("user");
    const wide = useMediaQuery("md");
    const catalog = useGalleryCatalog();
    const [source, setSource] = useState<Source>(() => (selected && isGalleryKind(selected.kind) ? selected.kind : "characters"));
    const [chosen, setChosen] = useState<IGalleryFilter>(NO_GALLERY_FILTER);
    const [tileSize, setTileSize] = useTileSize();
    const galleryKind: GalleryKind = source === "characters" ? "archive_pic" : source;
    const view = useGalleryView(galleryKind, catalog[galleryKind], chosen);
    const actions = galleryFilterActions(setChosen);
    const gallerySelected = useMemo(() => (selected && isGalleryKind(selected.kind) ? { kind: selected.kind, id: selected.id } : null), [selected]);
    const ready = source !== "characters" && view.status === "success";

    const label = (s: Source) => {
        switch (s) {
            case "characters":
                return t("profile.background.source.characters");
            case "archive_pic":
                return t("profile.background.gallery.source.archive_pic");
            case "story_cg":
                return t("profile.background.gallery.source.story_cg");
            case "story_scene":
                return t("profile.background.gallery.source.story_scene");
        }
    };
    const count = (s: Source) => (s === "characters" || catalog[s].status !== "success" ? null : catalog[s].tiles.length);
    const icon = (s: Source) => {
        const Icon = SOURCE_ICONS[s];
        return <Icon aria-hidden="true" />;
    };

    const sizeControl = (
        <ToggleGroup value={[tileSize]} onValueChange={(next) => next[0] && setTileSize(next[0] as TileSize)} variant="outline" size="sm" aria-label={t("profile.background.tileSize")} className="shrink-0 max-sm:hidden">
            <ToggleGroupItem className={SEGMENT_ITEM} value="s" aria-label={t("profile.background.tileSize.s")} title={t("profile.background.tileSize.s")}>
                S
            </ToggleGroupItem>
            <ToggleGroupItem className={SEGMENT_ITEM} value="m" aria-label={t("profile.background.tileSize.m")} title={t("profile.background.tileSize.m")}>
                M
            </ToggleGroupItem>
            <ToggleGroupItem className={SEGMENT_ITEM} value="l" aria-label={t("profile.background.tileSize.l")} title={t("profile.background.tileSize.l")}>
                L
            </ToggleGroupItem>
        </ToggleGroup>
    );
    const tiles = <GalleryTiles view={view} selected={gallerySelected} onPick={onPick} minTile={TILE_SIZES[tileSize]} />;
    const characters = (
        // The grids' shared picker scrolls its own tiles, so it gets the height under the strip and sticks there: scrolled to, it fills the view.
        <div className={`${STICKY_UNDER_STRIP} ${UNDER_STRIP} flex min-h-80 flex-col pt-4 [&>div]:max-sm:px-4`}>
            <CharacterArt selected={selected} onPick={onPick} />
        </div>
    );

    if (wide) {
        return (
            <div className="flex">
                <aside className={`${STICKY_UNDER_STRIP} ${UNDER_STRIP} flex w-64 shrink-0 flex-col gap-4 overflow-y-auto border-e px-3 pt-4 pb-4`} aria-label={t("profile.background.source")}>
                    <div className="flex flex-col gap-0.5">
                        <RailRow active={source === "characters"} onClick={() => setSource("characters")} name={label("characters")} icon={icon("characters")} />
                        <div className="my-1.5 border-t" />
                        <RailLabel>{t("profile.background.source.gallery")}</RailLabel>
                        {GALLERY_KINDS.map((kind) => (
                            <RailRow key={kind} active={source === kind} onClick={() => setSource(kind)} name={label(kind)} icon={icon(kind)} count={count(kind)} />
                        ))}
                    </div>
                    {ready && view.categories.length > 1 && (
                        <div>
                            <RailLabel>{t("profile.background.gallery.categories")}</RailLabel>
                            <div className="px-1">
                                <CategoryChips options={view.categories} chosen={view.filter.categories} onToggle={actions.toggleCategory} />
                            </div>
                        </div>
                    )}
                    {ready && (
                        <div className="flex min-h-0 flex-1 flex-col">
                            <RailLabel>{t("profile.background.gallery.stories")}</RailLabel>
                            <StoryPanel groups={view.groups} chosen={view.filter.group} total={view.groupTotal} onChoose={(group) => actions.update({ group })} />
                        </div>
                    )}
                </aside>
                <div className="min-w-0 flex-1">
                    {source === "characters" ? (
                        characters
                    ) : (
                        <>
                            <div className={`${STICKY_UNDER_STRIP} z-10 flex flex-col gap-2 border-b bg-background/80 px-6 pt-4 pb-2.5 backdrop-blur-md backdrop-saturate-150`}>
                                <GalleryResultsHeader view={view} query={chosen.query} actions={actions} end={sizeControl} />
                            </div>
                            <div className="px-6 pt-4 pb-10">{tiles}</div>
                        </>
                    )}
                </div>
            </div>
        );
    }

    return (
        <section className="flex flex-col">
            <div className={`${STICKY_UNDER_STRIP} z-20 flex flex-col gap-2.5 border-b bg-background/80 px-4 py-3 backdrop-blur-md backdrop-saturate-150`}>
                <div className="-mx-4 overflow-x-auto px-4 [scrollbar-width:none]">
                    <div className="inline-flex w-max gap-0.5 rounded-lg bg-muted/70 p-1" role="tablist" aria-label={t("profile.background.source")}>
                        {SOURCES.map((s) => {
                            const n = count(s);
                            return (
                                <button key={s} type="button" role="tab" aria-selected={source === s} onClick={() => setSource(s)} className={sourceTab(source === s)}>
                                    {icon(s)}
                                    {label(s)}
                                    {n !== null && <span className={COUNT}>{n}</span>}
                                </button>
                            );
                        })}
                    </div>
                </div>
                {source !== "characters" && <GalleryBar view={view} query={chosen.query} actions={actions} />}
            </div>
            {source === "characters" ? characters : <div className="px-4 pt-4 pb-10">{tiles}</div>}
        </section>
    );
});
