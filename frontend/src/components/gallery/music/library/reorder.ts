/**
 * A playlist's full id list with `fromId` moved to where `toId` stands: before
 * it when moving up, after it when moving down. Ids not shown in the list
 * (tracks the catalogue does not know right now) keep their places.
 */
export function moveTrack(ids: readonly string[], fromId: string, toId: string): string[] {
    const from = ids.indexOf(fromId);
    const to = ids.indexOf(toId);
    if (from < 0 || to < 0 || from === to) return [...ids];
    const out = ids.filter((id) => id !== fromId);
    const at = out.indexOf(toId);
    out.splice(from < to ? at + 1 : at, 0, fromId);
    return out;
}
