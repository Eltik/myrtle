import type { IContentAuditEntry } from "#/lib/api/admin";

/**
 * One clause of a content audit row's summary ("renamed tier S -> S+",
 * "moved 3 entries"), as data: `AuditTab` words each one through the
 * catalog and joins them. Pure, so the order and the folding are tested.
 */
export type ContentChange =
    | { kind: "listType"; listType: string }
    | { kind: "forkedFrom"; title: string }
    | { kind: "renamed"; from: string; to: string }
    | { kind: "description" }
    | { kind: "kinds"; count: number }
    | { kind: "listed"; listed: boolean }
    | { kind: "size"; size: string }
    | { kind: "cells"; count: number }
    | { kind: "tiersAdded"; names: string[] }
    | { kind: "tierRenamed"; from: string; to: string }
    | { kind: "tiersReordered"; count: number }
    | { kind: "tiersRestyled"; names: string[] }
    | { kind: "tiersDeleted"; names: string[]; dropped: number }
    | { kind: "entriesAdded"; count: number }
    | { kind: "entriesMoved"; count: number }
    | { kind: "entriesReordered"; count: number }
    | { kind: "entriesRemoved"; count: number }
    | { kind: "notesEdited"; count: number }
    | { kind: "published"; version: number; changelog: string | null }
    | { kind: "flair"; label: string | null }
    | { kind: "permission"; level: string }
    | { kind: "deleted"; tiers: number | null; entries: number | null }
    | { kind: "more"; count: number };

/** Tier renames listed one by one before the rest fold into "and N more". */
const RENAMES_SHOWN = 3;

/** The clauses of `entry`'s summary, in reading order. */
export function contentChanges(entry: IContentAuditEntry): ContentChange[] {
    const d = entry.details;
    const out: ContentChange[] = [];
    switch (entry.action) {
        case "create":
            if (d.list_type) out.push({ kind: "listType", listType: d.list_type });
            return out;
        case "fork":
            if (d.forked_from) out.push({ kind: "forkedFrom", title: d.forked_from });
            return out;
        case "publish":
            if (d.version != null) out.push({ kind: "published", version: d.version, changelog: d.changelog ?? null });
            return out;
        case "visibility":
            if (d.listed != null) out.push({ kind: "listed", listed: d.listed });
            return out;
        case "flair":
            out.push({ kind: "flair", label: d.flair ?? null });
            return out;
        case "grant":
        case "revoke":
            if (d.permission) out.push({ kind: "permission", level: d.permission });
            return out;
        case "delete":
            if (d.tiers != null || d.entries != null) out.push({ kind: "deleted", tiers: d.tiers ?? null, entries: d.entries ?? null });
            return out;
        case "edit":
            break;
    }

    if (d.renamed_from != null && d.renamed_from !== entry.target_name) out.push({ kind: "renamed", from: d.renamed_from, to: entry.target_name });
    if (d.description_changed) out.push({ kind: "description" });
    if (d.kinds) out.push({ kind: "kinds", count: d.kinds.length });
    if (d.listed != null) out.push({ kind: "listed", listed: d.listed });
    if (d.size) out.push({ kind: "size", size: d.size });
    if (d.cells_changed) out.push({ kind: "cells", count: d.cells_changed });

    if (d.tiers_added?.length) out.push({ kind: "tiersAdded", names: d.tiers_added.map((t) => t.name) });
    const renames = d.tiers_renamed ?? [];
    for (const r of renames.slice(0, RENAMES_SHOWN)) out.push({ kind: "tierRenamed", from: r.from, to: r.to });
    if (renames.length > RENAMES_SHOWN) out.push({ kind: "more", count: renames.length - RENAMES_SHOWN });
    if (d.tiers_reordered?.length) out.push({ kind: "tiersReordered", count: d.tiers_reordered.length });
    if (d.tiers_restyled?.length) out.push({ kind: "tiersRestyled", names: d.tiers_restyled.map((t) => t.name) });
    if (d.tiers_deleted?.length) out.push({ kind: "tiersDeleted", names: d.tiers_deleted.map((t) => t.name), dropped: d.entries_dropped ?? 0 });

    if (d.entries_added?.length) out.push({ kind: "entriesAdded", count: d.entries_added.length });
    if (d.entries_moved?.length) out.push({ kind: "entriesMoved", count: d.entries_moved.length });
    if (d.entries_reordered?.length) out.push({ kind: "entriesReordered", count: d.entries_reordered.length });
    if (d.entries_removed?.length) out.push({ kind: "entriesRemoved", count: d.entries_removed.length });
    if (d.notes_edited?.length) out.push({ kind: "notesEdited", count: d.notes_edited.length });
    return out;
}

/** The actor acted on content someone else owns: a moderator's action, or an editor's under a grant. */
export function actedOnOthers(entry: IContentAuditEntry): boolean {
    return entry.owner != null && entry.owner.user_id !== entry.actor.user_id;
}
