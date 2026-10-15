import type { ProfileHeader } from "#/types/generated/ProfileHeader";
import type { ProfileHeaderField } from "#/types/generated/ProfileHeaderField";
import type { ProfileLayout } from "#/types/generated/ProfileLayout";
import type { IUserProfile } from "#/types/user";

/** A number the profile header shows. The backend's `ProfileHeaderField` is the allow-list. */
export type HeaderField = ProfileHeaderField;

/** Every field, in the backend's canonical order, the order a stored `hidden` list keeps. */
export const HEADER_FIELDS = ["level", "registered", "sign_in", "operators", "skins", "items", "orundum", "originite", "lmd", "headhunting_permits", "ten_pull_permits"] as const satisfies readonly HeaderField[];

/** The fields that can be a stat strip card: every one but the level, the date and the sign-in tally. */
export const STRIP_CARDS = ["operators", "skins", "items", "orundum", "originite", "lmd", "headhunting_permits", "ten_pull_permits"] as const satisfies readonly HeaderField[];

/** The strip before it was editable, in its order: what a header with no `strip` shows. */
export const DEFAULT_STRIP: readonly HeaderField[] = ["operators", "skins", "items", "orundum", "originite"];

/** Most cards the strip holds, `HEADER_STRIP_MAX` on the backend. */
export const STRIP_MAX = 5;

export function isStripCard(field: HeaderField): boolean {
    return (STRIP_CARDS as readonly HeaderField[]).includes(field);
}

/**
 * Whether the owner hid `field` from visitors. With no header (the kill switch) nothing
 * is hidden. A visitor's copy carries the same list, so it also tells a hidden number
 * from one that is merely unsynced.
 */
export function isHidden(layout: ProfileLayout | null, field: HeaderField): boolean {
    return layout?.header?.hidden.includes(field) ?? false;
}

/**
 * The cards the strip renders, in order. With no header, the default five. The owner sees
 * every card they chose, hidden ones included (marked); a visitor's strip was already
 * resolved by the backend without its hidden cards, and the filter here only guards
 * against an owner-shaped payload.
 */
export function stripCards(layout: ProfileLayout | null, isOwner: boolean): HeaderField[] {
    const header = layout?.header;
    const cards = (header?.strip ?? DEFAULT_STRIP).filter(isStripCard);
    return isOwner ? cards : cards.filter((card) => !isHidden(layout, card));
}

/** The editor's working copy: the stored header made whole, the strip spelled out. */
export interface IHeaderDraft {
    hidden: HeaderField[];
    strip: HeaderField[];
}

export function headerDraft(layout: ProfileLayout | null): IHeaderDraft {
    return {
        hidden: HEADER_FIELDS.filter((field) => isHidden(layout, field)),
        strip: stripCards(layout, true),
    };
}

function sameList(a: readonly HeaderField[], b: readonly HeaderField[]): boolean {
    return a.length === b.length && a.every((v, i) => v === b[i]);
}

/** Whether `draft` is the header as it was before it was editable: nothing hidden, the default strip. */
export function isDefaultHeader(draft: IHeaderDraft): boolean {
    return draft.hidden.length === 0 && sameList(draft.strip, DEFAULT_STRIP);
}

/**
 * The `header` a save sends, and nothing else of the layout. `null` removes the key, so a
 * header put back to the default stores what a never-customized profile has. A strip
 * equal to the default is left out, so it keeps following the default.
 */
export function headerForSave(draft: IHeaderDraft): ProfileHeader | null {
    if (isDefaultHeader(draft)) return null;
    const hidden = HEADER_FIELDS.filter((field) => draft.hidden.includes(field));
    return sameList(draft.strip, DEFAULT_STRIP) ? { hidden } : { hidden, strip: [...draft.strip] };
}

export function toggleHidden(draft: IHeaderDraft, field: HeaderField): IHeaderDraft {
    const hidden = draft.hidden.includes(field) ? draft.hidden.filter((f) => f !== field) : [...draft.hidden, field];
    return { ...draft, hidden: HEADER_FIELDS.filter((f) => hidden.includes(f)) };
}

/** `draft` with `card` appended to the strip; unchanged when it is there, not a card, or the strip is full. */
export function addCard(draft: IHeaderDraft, card: HeaderField): IHeaderDraft {
    if (!isStripCard(card) || draft.strip.includes(card) || draft.strip.length >= STRIP_MAX) return draft;
    return { ...draft, strip: [...draft.strip, card] };
}

export function removeCard(draft: IHeaderDraft, card: HeaderField): IHeaderDraft {
    return { ...draft, strip: draft.strip.filter((c) => c !== card) };
}

/**
 * The grid classes for `count` cards. Five keeps the strip's classes from before it was
 * editable; fewer fill the row at desktop width instead of leaving empty columns.
 */
export function stripGridClass(count: number): string {
    switch (count) {
        case 1:
            return "grid-cols-1";
        case 2:
            return "grid-cols-2";
        case 3:
            return "grid-cols-2 sm:grid-cols-3";
        case 4:
            return "grid-cols-2 lg:grid-cols-4";
        default:
            return "grid-cols-2 sm:grid-cols-3 lg:grid-cols-5";
    }
}

/**
 * `profile` as the owner's header and strip render it with `draft` saved: the layout's
 * header replaced, every number kept (the owner is sent them all). For the editor's live
 * preview; the stored profile is untouched.
 */
export function previewHeader(profile: IUserProfile, draft: IHeaderDraft): IUserProfile {
    const layout: ProfileLayout = profile.profile_layout ?? { tabs: [] };
    return { ...profile, profile_layout: { ...layout, header: { hidden: [...draft.hidden], strip: [...draft.strip] } } };
}
