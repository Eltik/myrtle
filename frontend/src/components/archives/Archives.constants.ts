export const ARCHIVES_TABS = ["network", "dossiers", "numbers", "hidden"] as const;

export type ArchivesTab = (typeof ARCHIVES_TABS)[number];

export function isArchivesTab(value: unknown): value is ArchivesTab {
    return typeof value === "string" && (ARCHIVES_TABS as readonly string[]).includes(value);
}

/**
 * `?tab=` (absent is Network), `?id=` the open personnel file, `?chart=` the
 * open network chart (absent is the territories map).
 */
export interface IArchivesSearch {
    tab?: ArchivesTab;
    id?: string;
    chart?: string;
}

export function validateArchivesSearch(search: Record<string, unknown>): IArchivesSearch {
    const out: IArchivesSearch = {};
    if (isArchivesTab(search.tab) && search.tab !== "network") out.tab = search.tab;
    if (typeof search.id === "string" && search.id.trim() !== "") out.id = search.id.trim();
    if (typeof search.chart === "string" && search.chart.trim() !== "") out.chart = search.chart.trim();
    return out;
}
