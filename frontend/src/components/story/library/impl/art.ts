/**
 * WHICH PICTURE EACH SURFACE DRAWS, as a pure choice over the wire fields.
 *
 * The game ships four different images per story set and they are not
 * interchangeable (`docs/story-reader.md` section 2, "Story Collection art"):
 * `bannerUrl` is the KEY VISUAL, the authored plate the game's own Story
 * Collection draws; `titleImageUrl` is the 516x260 title LOGOTYPE, which is
 * the card's TITLE; `iconUrl` on a
 * group is the 56x68 chapter DECO glyph and exists on the 17 mainline chapters
 * alone; `coverUrl` is the Archives entry picture or a derived first
 * background, which is a different thing again and is what the page drew
 * before any of the three existed.
 *
 * Every rule here is a source CHOICE with its fallback, kept out of the
 * components so the branch that fires can be pinned in a test rather than read
 * off a screenshot.
 */

import type { LibGroup } from "./derive";

/** What a ticket prints as its TITLE: the authored logotype, or the group's name as text. */
export type ITitleSource = { kind: "logotype"; url: string } | { kind: "text" };

/**
 * THE TITLE OF A CARD IS THE GAME'S OWN LOGOTYPE. `act_view/variant_other/
 * image_title` in `ui/[uc]mixstory.ab` is a 280 x 138.4 slot on a 380 x 320
 * card and `stage_mix_story_overall_group_item_view/variant_other/image_title`
 * a 197 x 103 slot on a 240 x 180 one, both drawing `TitleImageId`, the
 * 516x260 plate whose typography is authored per event. So the answer to "use
 * a different font for the title matching the theme" is not a font at all: it
 * is this image, and it is on the wire for 81 of the 87 non-record groups.
 *
 * The 6 with none (`act17d0`, `act24side`, `act32side`, `act36side`,
 * `act45side`, `act13mini`) set the name as text instead, fitted by
 * {@link fitTitle}.
 */
export function titleSource(group: Pick<LibGroup, "titleImageUrl">): ITitleSource {
    return group.titleImageUrl ? { kind: "logotype", url: group.titleImageUrl } : { kind: "text" };
}

/** What a surface that draws the chapter's PICTURE draws: the key visual, the cover, or nothing. */
export type IPlateSource = { kind: "banner"; url: string } | { kind: "cover"; url: string } | { kind: "none" };

/**
 * THE PICTURE, for every surface that draws one: the list row's 172x92 box,
 * the ticket's rotated stub, and the palette the ticket is inked from.
 *
 * The key visual comes first and the cover second. They are never the same
 * file (81 groups carry both and 0 resolve to the same path), and the banner
 * is the authored plate where a `coverKind: "background"` cover is only the
 * first background of the chapter's first script. That derivation is worthless
 * on a chapter that opens on a fade: `main_9`, `main_12` and `main_14` derive
 * `bg_black.png` and `main_15` derives `bg_white.png`, so the stub was a black
 * rectangle and the SAMPLER, which reads the same file, found no ink in it and
 * let the padder invent a triad out of nothing. One choice serves all three
 * surfaces so the stub and the tape can never disagree about which picture the
 * card is about.
 */
export function plateSource(group: Pick<LibGroup, "bannerUrl" | "coverUrl">): IPlateSource {
    if (group.bannerUrl) return { kind: "banner", url: group.bannerUrl };
    if (group.coverUrl) return { kind: "cover", url: group.coverUrl };
    return { kind: "none" };
}

/**
 * WHERE THE KEY VISUAL IS CROPPED when it is the card's background.
 *
 * The game's own card cover-fits it dead centre: `image_ss_kv` is 282 x 204 on
 * a 240 x 180 card and `image_collect_kv` 237 x 204, both at `anchoredPos
 * (0,0)`, which is 113.0% and 112.9% of the size that would just cover, so the
 * game oversizes by 13.0% and biases nothing. Centre is therefore the rule.
 *
 * THE ONE EXCEPTION IS MAINLINE, and it is forced by the art rather than
 * chosen. The 64 event key visuals are 632x456 or 532x456 paintings with a
 * feathered vignette and NO text in them; the 17 mainline ones are 432x432
 * POSTERS with the chapter title typeset into the plate (`kv_stormwatch.png`
 * prints EP 09, then STORMWATCH across rows 0.65..0.78 of its height, then
 * COUNTY HILLOCK / VICTORIA EMPIRE at 0.85..0.95). The game never has to deal
 * with it because `variant_mainline` draws no logotype at all, only the
 * 180x180 poster contained by height plus the 51x64 deco. Ours draws the
 * logotype on every card, so a centred crop of a mainline poster would print
 * the chapter's name TWICE. Biasing to 20% of the overflow puts the visible
 * band at rows 0.091..0.635, which stops 1.5 points short of the typeset title
 * and keeps the crown and the EP numeral.
 */
export function plateCrop(group: Pick<LibGroup, "category">): string {
    return group.category === "main" ? "50% 20%" : "50% 50%";
}

/**
 * Which glyph a section's chip and heading draw. An ARC has its own 184x52
 * banner icon and it is the more specific of the three, so it beats the
 * shelf's; a shelf without an arc draws its 108x108 LOGO (`StorylineLogoId`,
 * 14 of 14 EN shelves), the emblem the game's Story Collection gives it, and
 * only a shelf with no logo falls to its 44x36 abbreviation; a section the
 * wire names none of these for draws the lucide glyph its id hashes to.
 *
 * The logo outranks the abbreviation since 2026-09-24: the bar drew "RL",
 * "UR", "LA" beside four arc logotypes, and a reader saw two kinds of mark and
 * asked for one. The abbreviation is still the ChapterModal's deco.
 */
export type ISectionIconSource = { kind: "arc"; url: string } | { kind: "logo"; url: string } | { kind: "shelf"; url: string } | { kind: "glyph" };

export function sectionIconSource(arcIcon: string | undefined, shelfLogo: string | undefined, shelfIcon: string | undefined): ISectionIconSource {
    if (arcIcon) return { kind: "arc", url: arcIcon };
    if (shelfLogo) return { kind: "logo", url: shelfLogo };
    if (shelfIcon) return { kind: "shelf", url: shelfIcon };
    return { kind: "glyph" };
}

/** True for the 108x108 shelf logo, which fills a larger square slot than the 44x36 abbreviation. */
export function iconIsLogo(source: ISectionIconSource): boolean {
    return source.kind === "logo";
}

/** True for the one icon shape that is a WIDE banner rather than a square-ish glyph, which is sized by height instead of by box. */
export function iconIsWide(source: ISectionIconSource): boolean {
    return source.kind === "arc";
}

/**
 * HOW A GAME GLYPH IS INKED ON A THEMED SURFACE. The art is monochrome and it
 * is drawn for the game's dark chrome, so it cannot be printed raw: measured
 * over the opaque pixels of the shipped PNGs, the 13 shelf abbreviations and
 * the 14 shelf logos have a mean luminance of 251.0 and 255.0, which is
 * INVISIBLE on a light page, while the 4 arc icons and the 17 chapter decos
 * sit at 90.9 to 114.9 and would be near-black on a dark one. No single
 * treatment survives both shapes, so the glyph is flattened to one ink and the
 * ink follows the theme: `brightness-0` collapses every one of them to black,
 * and `invert` lifts that back to white in dark mode.
 *
 * The TICKET is the exception and is not themed: it is dark art in both modes
 * (`GroupCard.module.css`), so its own glyph is inked there instead.
 */
export const GLYPH_INK = "brightness-0 opacity-60 dark:brightness-0 dark:invert dark:opacity-80";
