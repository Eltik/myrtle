/**
 * WHAT AN EXPORT WILL WEIGH, before anything is fetched.
 *
 * Words come from the index (`StoryEntry.wordCount`, the backend's own count).
 * Images come from a group's ILLUSTRATIONS listing when the caller has it
 * (every background and CG with the stories that name it, so the count is
 * exact and deduplicated the way the book deduplicates), and otherwise from
 * the group's `illustrationCount` times the measured CG share, scaled by how
 * much of the group is in scope. The fallback is WEAK: over the 449 EN groups
 * its CG count runs 0.474 to 1.695 of the real one (p10 to p90, median
 * 0.678), and the image bytes 0.938 to 1.623 with backgrounds (median 1.181).
 *
 * Bytes per image are MEANS (a sum is what is being estimated) of this
 * exporter's own output in Chrome (<= 1,200 px, JPEG at 0.8, PNG kept for
 * alpha) over a random 150 CGs and 150 backgrounds of the 2,241 distinct EN
 * images: a CG 135,820 B (median 89,550; 13 of 150 stayed PNG, mean of the
 * JPEGs alone 92,681), a background at book size 77,188 B (median 73,646,
 * none PNG; scenes now ship as 600 px thumbnails, see BACKGROUND_BYTES). The
 * plan's figures (165,358 and 129,763) were `sips` q80 medians and run high
 * against the browser's encoder. Text and the fixed part are main_0's own
 * EPUB: 34,000 deflated bytes of story XHTML over 6,872 words, and 146,818
 * bytes of everything else but the fonts, the 141,605-byte cover included
 * (register, 2026-09-30).
 */
import type { StoryIllustrations } from "#/types/generated/StoryIllustrations";
import { DEFAULT_WPM, minutesFor } from "../reading";
import { type BookIndex, scopeStories } from "./book";
import type { BookScope, ImageMode, Typeface } from "./types";

/** `pdf` is estimated with the EPUB's figures: the same JPEGs embed as they are, and the text is deflated either way (checked against main_0, register 2026-10-01). */
export type BookFormat = "epub" | "pdf" | "markdown" | "text";

/** A scene background is a THUMBNAIL (<= 600 px, JPEG 0.75): mean over the same 150 backgrounds, median 25,555, none PNG. */
export const BACKGROUND_BYTES = 26_311;
export const CG_BYTES = 135_820;
/** The share of a group's `illustrationCount` that is CGs, summed over the 449 EN groups: 1,431 of 5,908. */
export const CG_SHARE = 1_431 / 5_908;
/** Deflated story XHTML per index word, main_0: 34,000 / 6,872. */
export const EPUB_BYTES_PER_WORD = 4.95;
/** Uncompressed bytes per index word over all 449 EN groups: Markdown 30,819,369 / 4,360,146, text 29,466,994 / 4,360,146. */
export const MARKDOWN_BYTES_PER_WORD = 7.07;
export const TEXT_BYTES_PER_WORD = 6.76;
/** The EPUB's fixed cost: container, OPF, nav, NCX, CSS, colophon and the 1200x1600 cover. */
export const EPUB_FIXED_BYTES = 146_818;
export const FONT_BYTES: Record<Typeface, number> = { inter: 30_696 + 31_320, opendyslexic: 103_336 + 108_068, device: 0 };

export interface BookEstimate {
    stories: number;
    words: number;
    minutes: number;
    images: number;
    bytes: number;
    /** True when every image count came from an illustrations listing. */
    exactImages: boolean;
}

export interface EstimateOptions {
    images: ImageMode;
    format: BookFormat;
    typeface: Typeface;
    wpm?: number;
    /** Illustrations listings by group id, when the caller has them. */
    illustrations?: ReadonlyMap<string, StoryIllustrations | null | undefined>;
}

export function estimateBook(index: BookIndex, scope: BookScope, options: EstimateOptions): BookEstimate {
    const stories = scopeStories(index, scope);
    const words = stories.reduce((n, s) => n + (s.entry.wordCount ?? 0), 0);
    const packaged = options.format === "epub" || options.format === "pdf";
    const embeds = packaged && options.images !== "none";
    let cgs = 0;
    let backgrounds = 0;
    let exactImages = true;
    const byGroup = new Map<string, typeof stories>();
    for (const s of stories) byGroup.set(s.group.id, [...(byGroup.get(s.group.id) ?? []), s]);
    for (const [groupId, list] of byGroup) {
        const ids = new Set(list.map((s) => s.entry.id));
        const listing = options.illustrations?.get(groupId);
        if (listing) {
            const distinct = (items: StoryIllustrations["images"]) => new Set(items.filter((i) => i.url !== null && i.storyIds.some((id) => ids.has(id))).map((i) => i.url)).size;
            cgs += distinct(listing.images);
            backgrounds += distinct(listing.backgrounds);
            continue;
        }
        exactImages = false;
        const group = list[0].group;
        const readable = group.stories.filter((s) => s.hasScript).length || 1;
        const share = list.length / readable;
        const total = group.illustrationCount ?? 0;
        cgs += Math.round(total * CG_SHARE * share);
        backgrounds += Math.round(total * (1 - CG_SHARE) * share);
    }
    const images = options.images === "none" ? 0 : options.images === "cg" ? cgs : cgs + backgrounds;
    let bytes: number;
    if (packaged) {
        bytes = EPUB_FIXED_BYTES + FONT_BYTES[options.typeface] + Math.round(words * EPUB_BYTES_PER_WORD);
        if (embeds) bytes += cgs * CG_BYTES + (options.images === "cg+bg" ? backgrounds * BACKGROUND_BYTES : 0);
    } else {
        bytes = Math.round(words * (options.format === "markdown" ? MARKDOWN_BYTES_PER_WORD : TEXT_BYTES_PER_WORD));
    }
    return { stories: stories.length, words, minutes: minutesFor(words, options.wpm ?? DEFAULT_WPM), images, bytes, exactImages: options.images === "none" || exactImages };
}
