/**
 * An operator's archives dossier card, as data. Built from the
 * `/archives/operator/{id}` record (structured basic info, parsed by the
 * backend) by both the archives route's `head()` and the `archive` OG handler,
 * through {@link buildArchiveOgData}.
 *
 * The excerpt and the art are not in that record: the handler reads them from
 * `/operators/{id}` (the handbook's first prose section, and the elite art
 * path). Both are static game text for a fixed id, so they are not hashed; an
 * edited profile keeps its old card until the design version moves.
 */

import type { GamedataServer } from "#/lib/api/gamedata";
import { plainText } from "#/lib/gamedata/richtext";
import type { ArchiveOperator } from "#/types/generated/ArchiveOperator";

export interface IArchiveOgRow {
    label: string;
    value: string;
}

export interface IArchiveOgData {
    id: string;
    server: GamedataServer;
    name: string;
    rarity: number;
    /** The personnel number (`R001`, `PL03`); `null` when the file has none. */
    number: string | null;
    /** The first faction's name. */
    faction: string | null;
    /** Three or four basic-info rows, only fields the file actually has. */
    rows: IArchiveOgRow[];
    /** Filled by the OG handler only: the first prose record, rich text stripped and clamped. */
    excerpt?: string;
    artURL?: string;
}

/** At most this many basic-info rows; the panel has room for four. */
const MAX_ROWS = 4;

/**
 * The file's rows, best first, skipping any that only repeats the display
 * name: code name, gender, place of birth, race, then combat experience and
 * height to fill. A robot's file carries none of those and lists its own fields in
 * `extra` (model, manufacturer, set gender), which fill in the same way.
 */
export function archiveRows(file: ArchiveOperator): IArchiveOgRow[] {
    const p = file.profile;
    if (!p) return [];
    const rows: IArchiveOgRow[] = [];
    const add = (label: string, value: string | undefined) => {
        const v = value?.trim();
        if (v && v !== file.name.trim() && rows.length < MAX_ROWS) rows.push({ label, value: v });
    };
    add("Code name", p.codeName);
    add("Gender", p.gender);
    add("Place of birth", p.birthplace);
    add("Race", p.race);
    add("Combat exp.", p.combatExperience);
    add("Height", p.height);
    add("Manufacturer", p.manufacturer);
    for (const f of p.extra) add(f.label, f.value);
    return rows;
}

export function buildArchiveOgData(file: ArchiveOperator, server: GamedataServer): IArchiveOgData {
    return {
        id: file.id,
        server,
        name: file.name,
        rarity: file.rarity,
        number: file.number?.display ?? null,
        faction: file.factions[0]?.name ?? null,
        rows: archiveRows(file),
    };
}

/** The first prose record among handbook sections: the first whose text is not `[Label] value` lines (`【】` on CN). */
export function firstProseRecord(sections: readonly { stories: readonly { storyText: string }[] }[]): string | null {
    for (const section of sections) {
        const text = section.stories
            .map((s) => s.storyText)
            .join("\n")
            .trim();
        if (text && !/^[[【]/.test(text)) return plainText(text).replace(/\s+/g, " ").trim();
    }
    return null;
}

/** `text` cut to about `max` characters at a word (or, for CJK, character) boundary. */
export function clampExcerpt(text: string, max: number): string {
    if (text.length <= max) return text;
    const cut = text.slice(0, max);
    const space = cut.lastIndexOf(" ");
    return `${(space > max * 0.6 ? cut.slice(0, space) : cut).replace(/[,.;:，。、\s]+$/, "")}...`;
}

export function archiveOgHashParts(data: IArchiveOgData): unknown[] {
    return [data.id, data.server, data.name, data.rarity, data.number ?? "", data.faction ?? "", data.rows.map((r) => `${r.label}=${r.value}`).join("|")];
}
