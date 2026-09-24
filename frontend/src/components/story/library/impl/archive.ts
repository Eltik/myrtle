import { VOICE_LANGUAGE_ORDER } from "#/components/operators/detail/impl/constants";
import type { ArchiveClip } from "#/types/generated/ArchiveClip";
import type { ArchiveClipTrack } from "#/types/generated/ArchiveClipTrack";
import type { LangType } from "#/types/generated/LangType";
import type { StoryArchive } from "#/types/generated/StoryArchive";
import type { StoryArchiveSection } from "#/types/generated/StoryArchiveSection";

export type ArchiveKind = StoryArchiveSection["kind"];

/**
 * The order the sections are read in, which is NOT the order the wire sends
 * them: `act17side` arrives logs, landmarks, gallery and `act13side` news,
 * files, gallery, so a fixed order is the only way two archives read the same.
 *
 * It runs from the most text to the least: the logs and the landmarks are the
 * event's own writing, the news and the files are documents inside it, the
 * recordings are its voices, and the gallery and the music are the two shelves
 * a reader browses rather than reads. A kind the wire adds later is DROPPED
 * here rather than rendered blank, because a section with no renderer is a
 * heading over nothing.
 */
export const ARCHIVE_ORDER: readonly ArchiveKind[] = ["logs", "landmarks", "news", "files", "recordings", "gallery", "music"];

const RANK = new Map<string, number>(ARCHIVE_ORDER.map((kind, at) => [kind, at]));

/** The group's sections in reading order, and nothing at all while the query has no answer. */
export function archiveSections(archive: StoryArchive | null | undefined): StoryArchiveSection[] {
    if (!archive) return [];
    return archive.sections.filter((section) => RANK.has(section.kind)).sort((a, b) => (RANK.get(a.kind) ?? 0) - (RANK.get(b.kind) ?? 0));
}

/**
 * The rows the whole archive holds, summed over its sections.
 *
 * It is the LEAF count the backend already carries, so it counts logs rather
 * than the chapters they group into and clips rather than the nodes they hang
 * on, and the hidden clips of a recordings section are outside it because the
 * game's own shelf does not list them either.
 */
export function archiveTotal(sections: readonly StoryArchiveSection[]): number {
    return sections.reduce((sum, section) => sum + section.count, 0);
}

/** Whether the sheet shows the archive segment at all: an empty archive, a 404 and a query still in flight all read the same here. */
export function hasArchive(archive: StoryArchive | null | undefined): boolean {
    return archiveSections(archive).length > 0;
}

/**
 * One clip's tracks in the voices tab's own language order, so a reader who
 * knows that tab finds Japanese first here too. A language the order does not
 * name keeps its place at the end rather than being dropped.
 */
export function orderedTracks(clip: Pick<ArchiveClip, "tracks">): ArchiveClipTrack[] {
    const rank = (lang: LangType): number => {
        const at = (VOICE_LANGUAGE_ORDER as readonly string[]).indexOf(lang);
        return at === -1 ? VOICE_LANGUAGE_ORDER.length : at;
    };
    return [...clip.tracks].sort((a, b) => rank(a.language) - rank(b.language));
}

/** The language a clip opens on: the first the order names, or the first track there is. */
export function defaultLanguage(clip: Pick<ArchiveClip, "tracks">): LangType | null {
    return orderedTracks(clip)[0]?.language ?? null;
}
