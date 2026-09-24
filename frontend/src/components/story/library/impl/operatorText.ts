/**
 * Pure derivations for the operator dialog's four shelves: the handbook Files,
 * the Modules, the Voice Lines, and the counts their tab labels carry.
 *
 * Nothing here fetches or renders. The dialog hands each function whatever the
 * wire gave it, `null` while a query has not run, and gets back a list plus a
 * word count, so the counting is testable without a browser.
 *
 * A word is a whitespace-separated token of the TAG-FREE text: `plainText`
 * strips the game's rich-text spans first, because `<@ba.vup>12</>` is one word
 * to a reader and four to a naive split. The story tree is EN-only (section 1
 * of `docs/story-reader.md`), so a whitespace split is the right measure; a CJK
 * tree would need a segmenter and this would undercount it badly.
 */

import { plainText } from "#/lib/gamedata/richtext";
import type { IVoice, LangType } from "#/types/voices";

export function countWords(text: string | null | undefined): number {
    if (!text) return 0;
    const plain = plainText(text).trim();
    if (plain === "") return 0;
    return plain.split(/\s+/).length;
}

// =============================================================================
// Files: the handbook's `storyTextAudio` sections
// =============================================================================

/** The handbook shape this module needs, structurally: `HandbookItem.storyTextAudio`. */
export interface IHandbookSectionSource {
    storyTitle: string;
    stories: readonly { storyText: string }[];
}

export interface IFileSection {
    /** Stable per operator: the title, suffixed when the handbook repeats one. */
    key: string;
    title: string;
    /** Every paragraph of the section, joined as the handbook writes them. */
    text: string;
    words: number;
}

/**
 * One row per handbook section, in the handbook's own order, dropping a section
 * whose stories carry no text at all. A section holds more than one story on
 * the operators whose archive was extended after release, and they read as one
 * body of text, so they are JOINED rather than listed separately.
 */
export function fileSections(sections: readonly IHandbookSectionSource[] | null | undefined): IFileSection[] {
    if (!sections) return [];
    const used = new Map<string, number>();
    const out: IFileSection[] = [];
    for (const section of sections) {
        const text = section.stories
            .map((story) => story.storyText.trim())
            .filter((story) => story !== "")
            .join("\n\n");
        if (text === "") continue;
        const title = section.storyTitle.trim();
        const seen = used.get(title) ?? 0;
        used.set(title, seen + 1);
        out.push({ key: seen === 0 ? title : `${title}#${seen}`, title, text, words: countWords(text) });
    }
    return out;
}

// =============================================================================
// Modules: `uniEquipDesc`, which is the module's story, not its stat block
// =============================================================================

/** The module shape this module needs, structurally: `OperatorModule`. */
export interface IModuleSource {
    uniEquipId: string;
    uniEquipName: string;
    uniEquipDesc: string;
    uniEquipIcon: string;
    image: string | null;
    typeName1: string;
    typeName2: string | null;
    type: string;
    charEquipOrder: number;
}

export interface IModuleSection {
    key: string;
    name: string;
    /** `SOL-Y` when the module carries a two-part type designator, else `ORIGINAL`. */
    designator: string;
    description: string;
    words: number;
    iconPath: string;
    /** The operator's own kit rather than an unlockable module. */
    original: boolean;
}

/**
 * One row per module that carries description text, in the game's equip order.
 * The INITIAL module is KEPT: it is the operator's original gear and its
 * `uniEquipDesc` is a paragraph of story like any other, which is why the
 * reference site lists it. A module with an empty description is dropped rather
 * than drawn as an empty card.
 */
export function moduleSections(modules: readonly IModuleSource[] | null | undefined): IModuleSection[] {
    if (!modules) return [];
    return [...modules]
        .sort((a, b) => a.charEquipOrder - b.charEquipOrder || a.uniEquipId.localeCompare(b.uniEquipId))
        .flatMap((mod) => {
            const description = mod.uniEquipDesc.trim();
            if (description === "") return [];
            const designator = mod.typeName1 && mod.typeName2 ? `${mod.typeName1}-${mod.typeName2}` : mod.typeName1;
            return [
                {
                    key: mod.uniEquipId,
                    name: mod.uniEquipName,
                    designator,
                    description,
                    words: countWords(description),
                    iconPath: mod.image ?? `/textures/spritepack/ui_equip_big_img_hub_0/${mod.uniEquipIcon}.png`,
                    original: mod.type === "INITIAL",
                },
            ];
        });
}

// =============================================================================
// Voice lines
// =============================================================================

export interface IVoiceLine {
    key: string;
    title: string;
    text: string;
    words: number;
    /** The line's clip per language, so the player never re-scans `data`. */
    clips: Partial<Record<LangType, string>>;
}

/**
 * The operator's own lines, in the game's `voiceIndex` order.
 *
 * The filter is on `charWordId`, never on `charId`: every Amiya line carries
 * `charId: "char_002_amiya"` whatever the form, and only the key is prefixed
 * with the form's id, so a `charId` filter hands Guard Amiya the caster's
 * lines. This is the same rule the operator detail page's Audio tab applies.
 */
export function operatorVoiceLines(charWords: { [key in string]?: IVoice } | null | undefined, charId: string): IVoiceLine[] {
    if (!charWords || charId === "") return [];
    const prefix = `${charId}_`;
    const lines: { voice: IVoice; line: IVoiceLine }[] = [];
    for (const voice of Object.values(charWords)) {
        if (!voice || !voice.charWordId?.startsWith(prefix)) continue;
        const clips: Partial<Record<LangType, string>> = {};
        for (const datum of voice.data ?? []) {
            if (datum.language && datum.voiceUrl) clips[datum.language] = datum.voiceUrl;
        }
        lines.push({
            voice,
            line: { key: voice.id ?? voice.charWordId, title: voice.voiceTitle, text: voice.voiceText, words: countWords(voice.voiceText), clips },
        });
    }
    const sorted = lines.sort((a, b) => a.voice.voiceIndex - b.voice.voiceIndex || a.line.key.localeCompare(b.line.key)).map((row) => row.line);

    // An operator with a second voice PACK carries a second row per line under
    // its own `wordKey` (`char_102_texas_ITA_CN_002` beside
    // `char_102_texas_CN_002`), and on Texas the pair is identical in title, in
    // text and in all five clip URLs, so the shelf would print every line
    // twice. A row is dropped only when ALL of that matches one already kept,
    // never on the title alone: Kal'tsit also has two packs and not one of her
    // 76 rows is an exact twin.
    const seen = new Set<string>();
    return sorted.filter((line) => {
        const identity = JSON.stringify([line.title, line.text, Object.entries(line.clips).sort()]);
        if (seen.has(identity)) return false;
        seen.add(identity);
        return true;
    });
}

/** The languages at least one line ships, in the caller's display order. */
export function voiceLanguages(lines: readonly IVoiceLine[], order: readonly LangType[]): LangType[] {
    const present = new Set<LangType>();
    for (const line of lines) {
        for (const lang of Object.keys(line.clips) as LangType[]) present.add(lang);
    }
    const ordered = order.filter((lang) => present.has(lang));
    // A language the display order does not name still has clips, so it is
    // appended rather than dropped: the select is the only way to reach it.
    for (const lang of present) if (!ordered.includes(lang)) ordered.push(lang);
    return ordered;
}

// =============================================================================
// Tab counts
// =============================================================================

export type OperatorTabKey = "records" | "files" | "modules" | "voices";

export const OPERATOR_TAB_ORDER: readonly OperatorTabKey[] = ["records", "files", "modules", "voices"] as const;

export interface IOperatorTab {
    key: OperatorTabKey;
    /** Rows on the shelf, or `null` while its query has not answered. */
    count: number | null;
    /** Words on the shelf, or `null` when the shelf carries no text of its own (Records) or has not loaded. */
    words: number | null;
}

export interface ITabInput {
    /** Record stories, which the library index already carries, so this is never null. */
    records: number;
    files: readonly IFileSection[] | null;
    modules: readonly IModuleSection[] | null;
    voices: readonly IVoiceLine[] | null;
}

function sumWords(rows: readonly { words: number }[]): number {
    return rows.reduce((total, row) => total + row.words, 0);
}

/**
 * The four tab descriptors, counts and all.
 *
 * `null` is the honest answer for a shelf whose query has not run: the tab is
 * lazy, so a zero would claim the operator has no files when nobody has looked
 * yet. Records has no word count of its own, because a record's words live in
 * its script and the index does not carry them on this backend.
 */
export function operatorTabs(input: ITabInput): IOperatorTab[] {
    return [
        { key: "records", count: input.records, words: null },
        { key: "files", count: input.files?.length ?? null, words: input.files ? sumWords(input.files) : null },
        { key: "modules", count: input.modules?.length ?? null, words: input.modules ? sumWords(input.modules) : null },
        { key: "voices", count: input.voices?.length ?? null, words: input.voices ? sumWords(input.voices) : null },
    ];
}
