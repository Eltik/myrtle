import { useQuery } from "@tanstack/react-query";
import { ChevronDownIcon, PauseIcon, PlayIcon } from "lucide-react";
import type React from "react";
import { useMemo, useRef, useState } from "react";
import { asset } from "#/components/operators/detail/impl/assets";
import { VOICE_LANGUAGE_LABEL_KEY } from "#/components/operators/detail/impl/constants";
import type { messages as detailConstantsMessages } from "#/components/operators/detail/impl/constants.messages";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { storyIndexQueryOptions } from "#/lib/api/story";
import { GameText } from "#/lib/gamedata/GameText";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, getAvatarById } from "#/lib/utils";
import type { ArchiveClip } from "#/types/generated/ArchiveClip";
import type { ArchiveFile } from "#/types/generated/ArchiveFile";
import type { ArchiveLandmark } from "#/types/generated/ArchiveLandmark";
import type { ArchiveLogChapter } from "#/types/generated/ArchiveLogChapter";
import type { ArchiveNewsItem } from "#/types/generated/ArchiveNewsItem";
import type { ArchivePicture } from "#/types/generated/ArchivePicture";
import type { ArchiveRecordingNode } from "#/types/generated/ArchiveRecordingNode";
import type { ArchiveTrack } from "#/types/generated/ArchiveTrack";
import type { LangType } from "#/types/generated/LangType";
import type { StoryArchiveSection } from "#/types/generated/StoryArchiveSection";
import type { messages } from "./Archive.messages";
import { archiveTotal, defaultLanguage, orderedTracks } from "./archive";
import type { IChapterAudio } from "./ChapterAudio";
import { ArtLightbox, type IArtView } from "./IllustrationPanel";
import type { messages as artMessages } from "./IllustrationsTab.messages";

type ArchiveT = TypedT<typeof messages & typeof artMessages>;
type DetailT = TypedT<typeof detailConstantsMessages>;

export interface IArchivePanelProps {
    /** Already in reading order: `archiveSections` decided it, so the renderer has no order of its own. */
    sections: readonly StoryArchiveSection[];
    /** The sheet's one music channel, shared with the theme button on the hero. */
    audio: IChapterAudio;
}

/**
 * WHAT THE EVENT KEPT BESIDE ITS STORIES.
 *
 * The game's own Archives screen is seven shelves and no event carries more
 * than three of them, so this is a list of collapsible sections rather than a
 * second row of tabs: a tab strip with two live tabs and five dead ones would
 * be five promises the data cannot keep. The first section opens on arrival
 * and the rest are shut, because the counts are the index and the reader picks
 * from them.
 *
 * Every piece of prose here goes through `GameText`. The archive's text
 * carries the same `<@lv.item>` spans a script does, and rendering `{text}`
 * would print the tags; `dangerouslySetInnerHTML` would render them but hand
 * the page an HTML sink for table text.
 */
export function ArchivePanel({ sections, audio }: IArchivePanelProps): React.ReactElement {
    const t: ArchiveT = useT("story");
    const [view, setView] = useState<IArtView | null>(null);

    return (
        <div className="flex flex-col gap-2 px-4 py-3 sm:px-5">
            <p className="m-0 font-mono text-[10.5px] text-muted-foreground uppercase tabular-nums tracking-[0.07em]">{t("archive.total", { count: archiveTotal(sections) })}</p>
            {sections.map((section, at) => (
                <Section key={section.kind} section={section} first={at === 0} audio={audio} onView={setView} />
            ))}
            <ArtLightbox view={view} onClose={() => setView(null)} />
        </div>
    );
}

export type ChapterView = "entries" | "archive";

export const CHAPTER_VIEWS: readonly ChapterView[] = ["entries", "archive"];

/**
 * THE SECOND SEGMENT UNDER THE META ROW, and only where there is something
 * behind it.
 *
 * 6 of the 87 EN groups kept an archive, so on 81 of them this control is not
 * rendered at all rather than rendered disabled: a switch with one live side
 * is a decoration. While the query is in flight `sections` is empty for the
 * same reason it is empty for a group with no archive, which is deliberate: a
 * spinner here would announce a second view to every reader of every chapter
 * and then take it back from 81 of them.
 */
export function ChapterViewSwitch({ view, onView, sections }: { view: ChapterView; onView: (view: ChapterView) => void; sections: readonly StoryArchiveSection[] }): React.ReactElement | null {
    const t: ArchiveT = useT("story");
    if (sections.length === 0) return null;

    return (
        <fieldset className="msv-scroll flex shrink-0 gap-0.5 overflow-x-auto border-border border-b px-4 pb-3 sm:px-5" aria-label={t("archive.view.aria")}>
            <span className="flex gap-0.5 rounded-[9px] border border-border bg-secondary/45 p-0.75">
                {CHAPTER_VIEWS.map((key) => (
                    <button
                        key={key}
                        type="button"
                        onClick={() => onView(key)}
                        aria-pressed={view === key}
                        className={cn("h-11 pointer-coarse:h-11 shrink-0 cursor-pointer rounded-md px-3 font-sans font-semibold text-[11.5px] transition-colors sm:h-7", view === key ? "bg-background text-foreground shadow-sm/5" : "text-muted-foreground hover:text-foreground")}
                    >
                        {key === "entries" ? t("archive.view.entries") : t("archive.view.archive")}
                    </button>
                ))}
            </span>
        </fieldset>
    );
}

/** One shelf: a header that always shows its count, and a body that is mounted only while it is open. */
function Section({ section, first, audio, onView }: { section: StoryArchiveSection; first: boolean; audio: IChapterAudio; onView: (view: IArtView) => void }): React.ReactElement {
    const t: ArchiveT = useT("story");
    const [open, setOpen] = useState(first);

    return (
        <Collapsible open={open} onOpenChange={setOpen}>
            <CollapsibleTrigger className="flex min-h-11 w-full items-center justify-between gap-2 rounded-[9px] border border-border bg-secondary/35 px-3 py-2 text-left transition-colors hover:bg-secondary/55 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/60">
                <span className="flex min-w-0 items-baseline gap-2">
                    <span className="truncate font-sans font-semibold text-[13px] text-foreground">{t(`archive.section.${section.kind}`)}</span>
                    <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">{section.count}</span>
                </span>
                <ChevronDownIcon aria-hidden="true" className={cn("size-4 shrink-0 text-muted-foreground transition-transform duration-200 motion-reduce:transition-none", open && "rotate-180")} />
            </CollapsibleTrigger>
            <CollapsibleContent>
                <div className="pt-2 pb-1">
                    <Body section={section} audio={audio} onView={onView} />
                </div>
            </CollapsibleContent>
        </Collapsible>
    );
}

function Body({ section, audio, onView }: { section: StoryArchiveSection; audio: IChapterAudio; onView: (view: IArtView) => void }): React.ReactElement {
    switch (section.kind) {
        case "logs":
            return (
                <div className="flex flex-col gap-3">
                    {section.chapters.map((chapter) => (
                        <LogChapter key={chapter.id} chapter={chapter} />
                    ))}
                </div>
            );
        case "landmarks":
            return (
                <div className="grid grid-cols-1 gap-2.5 sm:grid-cols-2">
                    {section.landmarks.map((landmark) => (
                        <Landmark key={landmark.id} landmark={landmark} />
                    ))}
                </div>
            );
        case "news":
            return (
                <div className="flex flex-col gap-3">
                    {section.news.map((item) => (
                        <NewsItem key={item.id} item={item} />
                    ))}
                </div>
            );
        case "files":
            return (
                <div className="flex flex-col gap-3">
                    {section.files.map((file) => (
                        <FileCard key={file.id} file={file} />
                    ))}
                </div>
            );
        case "recordings":
            return <Recordings nodes={section.nodes} hidden={section.hidden} />;
        case "gallery":
            return (
                <div className="grid grid-cols-2 gap-2.5 sm:grid-cols-3">
                    {section.pictures.map((picture) => (
                        <GalleryTile key={picture.id} picture={picture} onView={onView} />
                    ))}
                </div>
            );
        case "music":
            return (
                <ul className="m-0 flex list-none flex-col gap-1.5 p-0">
                    {section.tracks.map((track) => (
                        <li key={track.id}>
                            <TrackRow track={track} audio={audio} />
                        </li>
                    ))}
                </ul>
            );
    }
}

/**
 * The logs of one chapter of the event, under the header the game prints for
 * it. `chapterIcon` is `NORMAL` or `SPECIAL` and nothing else, so it is a word
 * in a chip rather than art; a third value the table grows later prints itself
 * instead of disappearing.
 */
function LogChapter({ chapter }: { chapter: ArchiveLogChapter }): React.ReactElement {
    const t: ArchiveT = useT("story");
    const icon = chapter.chapterIcon === "NORMAL" ? t("archive.logs.icon.normal") : chapter.chapterIcon === "SPECIAL" ? t("archive.logs.icon.special") : chapter.chapterIcon;

    return (
        <section className="flex flex-col gap-1.5">
            <h4 className="m-0 flex flex-wrap items-center gap-2">
                {chapter.displayId ? <span className="rounded-[5px] border border-border bg-secondary/55 px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground tabular-nums">{chapter.displayId}</span> : null}
                <span className="font-sans font-semibold text-[13px] text-foreground">{chapter.name}</span>
                {icon ? <span className="font-mono text-[10px] text-muted-foreground uppercase tracking-[0.08em]">{icon}</span> : null}
            </h4>
            {chapter.unlockDesc ? (
                <p className="m-0 font-sans text-[11.5px] text-muted-foreground/85 leading-relaxed">
                    <GameText text={chapter.unlockDesc} />
                </p>
            ) : null}
            <ul className="m-0 flex list-none flex-col gap-1.5 p-0">
                {chapter.logs.map((log) => (
                    <li key={log.id} className="rounded-[9px] border border-border bg-secondary/25 px-3 py-2 font-sans text-[12.5px] text-foreground leading-relaxed">
                        <GameText text={log.text} />
                    </li>
                ))}
            </ul>
        </section>
    );
}

/** A place: its picture, the name the story uses, the name the setting writes on a map, and what it is. */
function Landmark({ landmark }: { landmark: ArchiveLandmark }): React.ReactElement {
    return (
        <article className="flex min-w-0 flex-col gap-1.5 overflow-hidden rounded-[10px] border border-border bg-secondary/25">
            {landmark.pictureUrl ? <img src={asset(landmark.pictureUrl)} alt="" aria-hidden="true" loading="lazy" decoding="async" className="aspect-16/9 w-full object-cover" /> : null}
            <div className="flex flex-col gap-1 px-3 pt-1 pb-3">
                <span className="font-sans font-semibold text-[13px] text-foreground">{landmark.name}</span>
                {landmark.engName ? <span className="font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.06em]">{landmark.engName}</span> : null}
                <p className="m-0 font-sans text-[12px] text-muted-foreground leading-relaxed">
                    <GameText text={landmark.description} />
                </p>
            </div>
        </article>
    );
}

/** One in-setting news story, under the masthead it ran on. */
function NewsItem({ item }: { item: ArchiveNewsItem }): React.ReactElement {
    const t: ArchiveT = useT("story");
    const masthead = item.format?.mainLogoUrl ?? item.format?.logoUrl;

    return (
        <article className="flex flex-col gap-2 rounded-[10px] border border-border bg-secondary/25 px-3 py-3">
            <header className="flex flex-col gap-1">
                {masthead ? <img src={asset(masthead)} alt={item.format?.typeName ?? ""} loading="lazy" decoding="async" className="h-7 w-auto max-w-full self-start object-contain" /> : null}
                <span className="flex flex-wrap items-center gap-x-2 font-mono text-[10px] text-muted-foreground uppercase tracking-[0.08em]">
                    {item.format?.typeName && !masthead ? <span>{item.format.typeName}</span> : null}
                    {item.author ? <span>{t("archive.news.author", { author: item.author })}</span> : null}
                </span>
                <h4 className="m-0 font-sans font-semibold text-[13.5px] text-foreground leading-snug">
                    <GameText text={item.title} />
                </h4>
            </header>
            <div className="flex flex-col gap-1.5">
                {item.lines.map((line, at) =>
                    line.kind === "text" ? (
                        // biome-ignore lint/suspicious/noArrayIndexKey: a line's position in its own story is its identity; the table gives it no id
                        <p key={at} className="m-0 font-sans text-[12.5px] text-foreground/90 leading-relaxed">
                            <GameText text={line.text} />
                        </p>
                    ) : line.url ? (
                        // biome-ignore lint/suspicious/noArrayIndexKey: same
                        <img key={at} src={asset(line.url)} alt={line.name} loading="lazy" decoding="async" className="w-full rounded-[7px] object-contain" />
                    ) : null,
                )}
            </div>
        </article>
    );
}

/** One document: the drawn title where the event has one, the written one where it does not, and the date it is stamped with. */
function FileCard({ file }: { file: ArchiveFile }): React.ReactElement {
    return (
        <article className="flex flex-col gap-2 rounded-[10px] border border-border bg-secondary/25 px-3 py-3">
            {file.titlePictureUrl ? <img src={asset(file.titlePictureUrl)} alt={file.title} loading="lazy" decoding="async" className="h-9 w-auto max-w-full self-start object-contain" /> : <h4 className="m-0 font-sans font-semibold text-[13.5px] text-foreground leading-snug">{file.title}</h4>}
            {file.date ? <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">{file.date}</span> : null}
            {file.pictureUrl ? <img src={asset(file.pictureUrl)} alt="" aria-hidden="true" loading="lazy" decoding="async" className="w-full rounded-[7px] object-contain" /> : null}
            <p className="m-0 whitespace-pre-line font-sans text-[12.5px] text-foreground/90 leading-relaxed">
                <GameText text={file.text} />
            </p>
        </article>
    );
}

/** One picture, opening the SAME lightbox the Illustrations tab opens. */
function GalleryTile({ picture, onView }: { picture: ArchivePicture; onView: (view: IArtView) => void }): React.ReactElement {
    const t: ArchiveT = useT("story");
    const url = picture.url ? asset(picture.url) : null;

    return (
        <figure className="m-0 flex min-w-0 flex-col gap-1">
            {url ? (
                <button
                    type="button"
                    onClick={() => onView({ name: picture.title, url })}
                    aria-label={t("illustrations.item.view", { name: picture.title })}
                    className="block aspect-16/9 w-full cursor-pointer overflow-hidden rounded-[10px] border border-border bg-secondary/55 ring-inset transition-colors hover:border-primary/45 focus-visible:ring-2 focus-visible:ring-ring/60"
                >
                    <img src={url} alt="" aria-hidden="true" loading="lazy" decoding="async" className="h-full w-full object-cover" />
                </button>
            ) : (
                <div className="flex aspect-16/9 w-full items-center justify-center rounded-[10px] border border-border border-dashed bg-secondary/25 px-2 text-center font-sans text-[10.5px] text-muted-foreground/70">{t("illustrations.item.notExtracted")}</div>
            )}
            <figcaption className="flex min-w-0 flex-col gap-0.5">
                <span className="truncate font-sans text-[12px] text-foreground" title={picture.title}>
                    {picture.title}
                </span>
                {picture.description ? (
                    <span className="font-sans text-[11px] text-muted-foreground leading-relaxed">
                        <GameText text={picture.description} />
                    </span>
                ) : null}
            </figcaption>
        </figure>
    );
}

/** One track, played through the sheet's own channel so it and the hero's theme can never sound at once. */
function TrackRow({ track, audio }: { track: ArchiveTrack; audio: IChapterAudio }): React.ReactElement {
    const t: ArchiveT = useT("story");
    const name = track.name?.trim() ? track.name : t("archive.music.untitled");
    const key = `track:${track.id}`;
    const playing = audio.playing === key;

    return (
        <div className="flex min-h-11 items-center gap-2.5 rounded-[9px] border border-border bg-secondary/25 px-2.5 py-1.5">
            <button
                type="button"
                onClick={() => audio.toggle(key, { intro: track.introUrl ?? undefined, loop: track.loopUrl }, track.name?.trim() ? track.name : null)}
                aria-label={playing ? t("archive.theme.pause", { name }) : t("archive.theme.play", { name })}
                className="flex size-11 shrink-0 cursor-pointer items-center justify-center rounded-full border border-border bg-background text-foreground transition-colors hover:border-primary/45 hover:text-primary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/60 sm:size-9"
            >
                {playing ? <PauseIcon className="size-4" aria-hidden="true" /> : <PlayIcon className="size-4" aria-hidden="true" />}
            </button>
            <span className="flex min-w-0 flex-col">
                <span className="truncate font-sans text-[12.5px] text-foreground">{name}</span>
                {track.description ? (
                    <span className="truncate font-sans text-[11px] text-muted-foreground">
                        <GameText text={track.description} />
                    </span>
                ) : null}
            </span>
        </div>
    );
}

/**
 * The event's voices, in the nodes the game unlocks them in, plus the clips it
 * hides until then. The hidden ones belong to no node (8 of `main_14`'s 43),
 * so they are shown under their own heading rather than quietly appended to
 * the last node, which would claim they unlock with it.
 */
function Recordings({ nodes, hidden }: { nodes: readonly ArchiveRecordingNode[]; hidden: readonly ArchiveClip[] }): React.ReactElement {
    const t: ArchiveT = useT("story");
    const player = useClipPlayer();
    const nameOf = useOperatorNames();

    return (
        <div className="flex flex-col gap-3">
            {/* biome-ignore lint/a11y/useMediaCaption: game voice clips ship no transcript track */}
            <audio ref={player.ref} onEnded={player.onEnded} className="hidden" />
            {nodes.map((node) => (
                <section key={node.id} className="flex flex-col gap-1.5">
                    <h4 className="m-0 font-sans font-semibold text-[13px] text-foreground">{node.title}</h4>
                    {node.unlockDesc ? (
                        <p className="m-0 font-sans text-[11.5px] text-muted-foreground/85 leading-relaxed">
                            <GameText text={node.unlockDesc} />
                        </p>
                    ) : null}
                    <ul className="m-0 flex list-none flex-col gap-1.5 p-0">
                        {node.clips.map((clip) => (
                            <li key={`${clip.charId}-${clip.voiceId}-${clip.index}`}>
                                <ClipRow clip={clip} player={player} name={nameOf(clip.charId)} />
                            </li>
                        ))}
                    </ul>
                </section>
            ))}
            {hidden.length > 0 ? (
                <section className="flex flex-col gap-1.5">
                    <h4 className="m-0 font-sans font-semibold text-[13px] text-foreground">{t("archive.recordings.hidden")}</h4>
                    <p className="m-0 font-sans text-[11.5px] text-muted-foreground/85 leading-relaxed">{t("archive.recordings.hiddenNote")}</p>
                    <ul className="m-0 flex list-none flex-col gap-1.5 p-0">
                        {hidden.map((clip) => (
                            <li key={`${clip.charId}-${clip.voiceId}-${clip.index}`}>
                                <ClipRow clip={clip} player={player} name={nameOf(clip.charId)} />
                            </li>
                        ))}
                    </ul>
                </section>
            ) : null}
        </div>
    );
}

interface IClipPlayer {
    ref: React.RefObject<HTMLAudioElement | null>;
    playingUrl: string | null;
    play: (url: string) => void;
    stop: () => void;
    onEnded: () => void;
}

/** ONE element for every clip in the section, so a second press stops the first without a stop button anywhere. */
function useClipPlayer(): IClipPlayer {
    const ref = useRef<HTMLAudioElement | null>(null);
    const [playingUrl, setPlayingUrl] = useState<string | null>(null);

    const stop = (): void => {
        ref.current?.pause();
        setPlayingUrl(null);
    };

    const play = (url: string): void => {
        const element = ref.current;
        if (!element) return;
        if (playingUrl === url) {
            stop();
            return;
        }
        element.src = url;
        setPlayingUrl(url);
        void element.play().catch(() => setPlayingUrl(null));
    };

    return { ref, playingUrl, play, stop, onEnded: () => setPlayingUrl(null) };
}

/**
 * One clip: who speaks, in which language, and a press to hear it.
 *
 * The language picker is the voices tab's own vocabulary, ordered by
 * `VOICE_LANGUAGE_ORDER` and labelled from `VOICE_LANGUAGE_LABEL_KEY`, so a
 * reader who knows that tab reads this one. Switching language while a clip is
 * sounding stops it: the two files are different recordings, not two tracks of
 * one.
 */
function ClipRow({ clip, player, name }: { clip: ArchiveClip; player: IClipPlayer; name: string }): React.ReactElement {
    const t: ArchiveT = useT("story");
    const td: DetailT = useT("operators");
    const tracks = orderedTracks(clip);
    const [language, setLanguage] = useState<LangType | null>(() => defaultLanguage(clip));
    const chosen = tracks.find((track) => track.language === language) ?? tracks[0];
    const url = chosen ? asset(chosen.url) : null;
    const playing = url !== null && player.playingUrl === url;

    return (
        <div className="flex min-h-11 flex-wrap items-center gap-2 rounded-[9px] border border-border bg-secondary/25 px-2.5 py-1.5">
            <button
                type="button"
                disabled={url === null}
                onClick={() => url && player.play(url)}
                aria-label={playing ? t("archive.clip.pause", { name }) : t("archive.clip.play", { name })}
                className="flex size-11 shrink-0 cursor-pointer items-center justify-center rounded-full border border-border bg-background text-foreground transition-colors hover:border-primary/45 hover:text-primary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/60 disabled:cursor-default disabled:opacity-45 sm:size-9"
            >
                {playing ? <PauseIcon className="size-4" aria-hidden="true" /> : <PlayIcon className="size-4" aria-hidden="true" />}
            </button>
            <img src={getAvatarById(clip.charId)} alt="" aria-hidden="true" loading="lazy" decoding="async" className="size-7 shrink-0 rounded-full bg-secondary/55 object-cover" />
            <span className="min-w-0 flex-1 basis-24 truncate font-sans text-[12.5px] text-foreground">{name}</span>
            {tracks.length > 1 ? (
                <select
                    aria-label={t("archive.clip.language")}
                    value={chosen?.language ?? ""}
                    onChange={(e) => {
                        if (playing) player.stop();
                        setLanguage(e.target.value as LangType);
                    }}
                    className="h-11 pointer-coarse:h-11 shrink-0 cursor-pointer rounded-[7px] border border-border bg-background px-1.5 font-mono text-[10.5px] text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/60 sm:h-8"
                >
                    {tracks.map((track) => (
                        <option key={track.language} value={track.language}>
                            {VOICE_LANGUAGE_LABEL_KEY[track.language] ? td(VOICE_LANGUAGE_LABEL_KEY[track.language]) : track.language}
                        </option>
                    ))}
                </select>
            ) : null}
        </div>
    );
}

/**
 * The speaking operators' names, off the library index the sheet was opened
 * from. The archive names a clip by `charId` alone and the index is already in
 * the cache here, so one subscription serves all 43 of `main_14`'s clips;
 * where the index knows no such operator the id itself is printed rather than
 * a blank.
 */
function useOperatorNames(): (charId: string) => string {
    const server = useGamedataServer();
    const index = useQuery({ ...storyIndexQueryOptions(server), notifyOnChangeProps: ["data"] });
    const names = useMemo(() => new Map((index.data?.records ?? []).map((record) => [record.charId, record.name])), [index.data]);
    return (charId: string) => names.get(charId) ?? charId;
}
