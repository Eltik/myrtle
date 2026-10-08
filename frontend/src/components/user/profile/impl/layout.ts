import type { ProfileLayout } from "#/types/generated/ProfileLayout";
import type { ProfileTab } from "#/types/generated/ProfileTab";
import { isTabId, TAB_IDS, type TabId } from "./types";

/**
 * Which tab-memory rule the page runs. `stored`: the last tab opened, remembered per
 * browser across every profile. `visitor` opens a
 * customized profile on its owner's first visible tab and remembers a pick only for
 * the visit, so one owner's arrangement never leaks into the next profile read.
 */
export type TabMemory = "stored" | "visitor";

/**
 * `stored` whenever the owner never customized (`layout` null, the kill switch) or
 * the reader is the owner.
 */
export function tabMemory(layout: ProfileLayout | null, isOwner: boolean): TabMemory {
    return layout === null || isOwner ? "stored" : "visitor";
}

/** The canonical layout: every tab, in `TAB_IDS` order, visible. */
export function defaultLayout(): ProfileTab[] {
    return TAB_IDS.map((id) => ({ id, visible: true }));
}

/** How many showcase blocks `layout` carries. A visitor's copy is already pruned by the backend. */
export function showcaseBlockCount(layout: ProfileLayout | null): number {
    return layout?.showcase?.blocks.length ?? 0;
}

/** `tabs` with unknown ids and repeats dropped, the first of a repeat kept. */
function knownTabs(tabs: readonly { id: string; visible: boolean }[] | null | undefined): ProfileTab[] {
    const out: ProfileTab[] = [];
    for (const tab of tabs ?? []) {
        if (!isTabId(tab.id) || out.some((t) => t.id === tab.id)) continue;
        out.push({ id: tab.id, visible: tab.visible });
    }
    return out;
}

/**
 * The client's copy of the backend's normalization: unknown ids and repeats dropped
 * (the first wins), missing tabs appended visible in canonical order, except a
 * missing Showcase, which goes first. The backend
 * already sends the owner a normalized layout; this keeps the editor's working copy
 * whole. Never apply it to a visitor's layout, whose private tabs are absent on
 * purpose and would come back visible.
 */
export function normalizeLayout(tabs: readonly { id: string; visible: boolean }[] | null | undefined): ProfileTab[] {
    const out = knownTabs(tabs);
    for (const id of TAB_IDS) {
        if (out.some((t) => t.id === id)) continue;
        if (id === "showcase") out.unshift({ id, visible: true });
        else out.push({ id, visible: true });
    }
    return out;
}

/**
 * The tabs the bar renders, in order. With no layout, every tab in canonical order.
 * The owner sees every tab, private ones included, so they can open and edit them. A
 * visitor sees the visible ones: the backend has already removed the rest from the
 * layout it sent, so a tab ABSENT from the list stays hidden, and the `visible`
 * filter only guards against an owner-shaped payload.
 *
 * Showcase is the exception for a visitor: it is a tab only while it has a block. So
 * with no layout (the kill switch) a visitor's bar has no Showcase, and the owner's has
 * Showcase first, where they set it up.
 */
export function shownTabs(layout: ProfileLayout | null, isOwner: boolean): TabId[] {
    const tabs = layout === null ? defaultLayout() : knownTabs(layout.tabs);
    const visible = isOwner ? tabs : tabs.filter((t) => t.visible && (t.id !== "showcase" || showcaseBlockCount(layout) > 0));
    return visible.map((t) => t.id);
}

/**
 * The tab to show. Under the `stored` rule that is the remembered tab.
 * A visitor gets their pick from this visit, or the owner's first visible tab when
 * they have not picked yet or their pick is not a tab this profile shows. `null`
 * when the profile shows no tab at all.
 */
export function resolveActiveTab(shown: readonly TabId[], memory: TabMemory, stored: TabId, picked: TabId | null): TabId | null {
    const first = shown[0] ?? null;
    const remembered = memory === "stored" ? stored : picked;
    return remembered !== null && shown.includes(remembered) ? remembered : first;
}

/** Whether `tabs` is the canonical layout, which saving stores as "no layout". */
export function isDefaultLayout(tabs: readonly ProfileTab[]): boolean {
    return tabs.length === TAB_IDS.length && tabs.every((t, i) => t.id === TAB_IDS[i] && t.visible);
}

/**
 * The `profile_layout` a tab editor save sends: the `tabs` key and nothing else, so the
 * stored showcase and header background are kept as they are (a block that has since
 * gone dangling would otherwise fail the save). `null`, "no layout", only after the
 * owner pressed Reset (`reset`) and only when it loses nothing: canonical tabs, no
 * showcase block and no background in `layout`, the owner's stored layout.
 */
export function tabsForSave(tabs: readonly ProfileTab[], reset: boolean, layout: ProfileLayout | null): { tabs: ProfileTab[] } | null {
    if (reset && isDefaultLayout(tabs) && showcaseBlockCount(layout) === 0 && !layout?.background) return null;
    return { tabs: [...tabs] };
}

/**
 * `items` with the entry at `index` moved by `delta` places, clamped to the ends, and the
 * index it lands at; `null` when nothing moves (already at that end, or no such entry).
 * Any list: the tab editor's tabs, showcase blocks and a block's favourites all move so.
 */
export function moveEntry<T>(items: readonly T[], index: number, delta: number): { next: T[]; to: number } | null {
    const to = Math.max(0, Math.min(items.length - 1, index + delta));
    if (to === index || index < 0 || index >= items.length) return null;
    const next = [...items];
    const [moved] = next.splice(index, 1);
    next.splice(to, 0, moved);
    return { next, to };
}

/** An unchanged copy when nothing moves. */
export function moveTab<T>(tabs: readonly T[], index: number, delta: number): T[] {
    return moveEntry(tabs, index, delta)?.next ?? [...tabs];
}

export function toggleTab(tabs: readonly ProfileTab[], id: TabId): ProfileTab[] {
    return tabs.map((t) => (t.id === id ? { ...t, visible: !t.visible } : t));
}
