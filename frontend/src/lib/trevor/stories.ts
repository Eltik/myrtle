/**
 * Which cited source is a story the reader can open, and what to call it.
 *
 * A citation's `storyId` is a corpus source id: most are story ids, but an
 * answer can also cite an operator file or a profile (`archive_char_...`,
 * `profile_...`), which the reader has no page for. The story index the
 * reader route already loaded is the arbiter: a source it lists is a link to
 * `/stories/$storyId?line=`, anything else is shown as a quote with no link.
 */
import type { StoryIndex } from "#/types/generated/StoryIndex";

export interface IStoryLabel {
    name: string;
    groupName: string;
    code: string | null;
}

export function storyLabel(index: StoryIndex | null | undefined, storyId: string): IStoryLabel | null {
    if (!index) return null;
    for (const g of index.groups) {
        const s = g.stories.find((x) => x.id === storyId);
        if (s) {
            const rec = g.category === "record" ? index.records.find((r) => r.stories.some((x) => x.id === storyId)) : undefined;
            return { name: s.name, groupName: rec?.name ?? g.name, code: s.code ?? null };
        }
    }
    for (const r of index.records) {
        const s = r.stories.find((x) => x.id === storyId);
        if (s) return { name: s.name, groupName: r.name, code: s.code ?? null };
    }
    return null;
}
