/** `/announcements` search params: the board (`server`) and the open notice (`id`). Both deep-link. */
export interface IAnnouncementsSearch {
    server?: string;
    id?: string;
}

export function validateAnnouncementsSearch(search: Record<string, unknown>): IAnnouncementsSearch {
    const out: IAnnouncementsSearch = {};
    if (typeof search.server === "string" && /^[a-z]{2,8}$/.test(search.server)) out.server = search.server;
    if (typeof search.id === "string" && search.id.trim() !== "") out.id = search.id.trim();
    else if (typeof search.id === "number" && Number.isFinite(search.id)) out.id = String(search.id);
    return out;
}
