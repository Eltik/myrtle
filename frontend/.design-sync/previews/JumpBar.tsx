import { JumpBar, LibrarySectionHead, LibraryToolbarButton } from "frontend";
import type React from "react";
import { useState } from "react";
import type { IBrowseToolbarState } from "../../src/components/story/library/impl/BrowseToolbar";
import type { LibrarySort, ReadFilter } from "../../src/components/story/library/impl/derive";
import type { FilterKey, ViewMode } from "../../src/components/story/library/impl/sections";


// JumpBar is the Archives' one pinned row: a scroller of section chips (the
// first, active one expanded to its name, range and count in a primary pill;
// the rest collapsed to their act banner or shelf logo, or a mono
// abbreviation), fading at whichever edge has chips behind it, and ending in
// the "Search and filters" button. Under 640 the scroller becomes a section
// picker pill. The chips are the live EN sections.

const ICON = (path: string) => `/textures/spritepack/${path}.png`;

// The Archives' sections as Browse builds them from the live EN storylines:
// the four mainline arcs (wide act banners, a chapter run each), the themed
// shelves (108x108 logos, mixed kinds, "includes" mainline chapters where they
// hold any), the unmarked "Other events" and the operator records.
const ARC_1: IChipModel = { id: "arc-mainLine-9", range: { from: 0, to: 3 }, includes: null, name: "HOUR OF AN AWAKENING", count: 4, iconUrl: ICON("mixstory_deco_sprites_h2_0/act_0"), iconWide: true, iconLogo: false, glyph: 0, filter: "main" };
const ARC_2: IChipModel = { id: "arc-mainLine-101", range: { from: 4, to: 8 }, includes: null, name: "SHATTER OF A VISION", count: 6, iconUrl: ICON("mixstory_deco_sprites_h2_0/act_1"), iconWide: true, iconLogo: false, glyph: 0, filter: "main" };
const ARC_3: IChipModel = { id: "arc-mainLine-201", range: { from: 9, to: 14 }, includes: null, name: "SHADOW OF A DYING SUN", count: 9, iconUrl: ICON("mixstory_deco_sprites_h2_0/act_2"), iconWide: true, iconLogo: false, glyph: 0, filter: "main" };
const ARC_4: IChipModel = { id: "arc-mainLine-301", range: { from: 15, to: 16 }, includes: null, name: "NEXUS POINT OF FUTURE", count: 3, iconUrl: ICON("mixstory_deco_sprites_h2_0/act_3"), iconWide: true, iconLogo: false, glyph: 0, filter: "main" };
const shelf = (id: string, name: string, abbr: string, count: number, includes: { from: number; to: number } | null = null, filter: IChipModel["filter"] = "events"): IChipModel => ({ id: `line-${id}`, range: null, includes, name, count, iconUrl: ICON(`mixstory_logo_sprites_0/storyline_${abbr}`), iconWide: false, iconLogo: true, glyph: 1, filter });
const THE_ARK = shelf("ssLine_1", "The Ark", "Rl", 9, { from: 7, to: 14 }, null);
const WILDFIRE = shelf("ssLine_13", "Wildfire", "Ur", 5, { from: 1, to: 16 }, null);
const SNOW = shelf("ssLine_3", "Snow and Silver Steel", "Kj", 4);
const SETTE_COLLI = shelf("ssLine_4", "Sette Colli's Sprouts", "Si", 4);
const NEON = shelf("ssLine_5", "Under the Neon", "Ka", 4);
const AGES = shelf("ssLine_6", "Through the Ages", "Su", 9);
const DEPTHS = shelf("ssLine_8", "Glimpse of the Depths", "Ae", 7, { from: 14, to: 14 }, null);
const SUMMER = shelf("ssLine_11", "Summertime Beats", "St", 7);
const OTHER: IChipModel = { id: "other", range: null, includes: null, name: "Other events", abbr: "MISC", count: 31, glyph: 6, filter: null };
const RECORDS: IChipModel = { id: "records", range: null, includes: null, name: "Operator records", abbr: "REC", count: 319, glyph: 3, filter: null };

/** The toolbar state Browse owns, held locally so every control in the story works. */
function useToolbarState(initial: Partial<Pick<IBrowseToolbarState, "query" | "filter" | "readFilter" | "sort" | "view">> = {}): IBrowseToolbarState {
    const [query, setQuery] = useState(initial.query ?? "");
    const [filter, setFilter] = useState<FilterKey>(initial.filter ?? "all");
    const [readFilter, setReadFilter] = useState<ReadFilter>(initial.readFilter ?? "any");
    const [sort, setSort] = useState<LibrarySort>(initial.sort ?? "default");
    const [view, setView] = useState<ViewMode>(initial.view ?? "grid");
    return { query, setQuery, filter, setFilter, readFilter, setReadFilter, sort, setSort, view, setView };
}

const ALL = [ARC_1, ARC_2, ARC_3, ARC_4, THE_ARK, WILDFIRE, SNOW, SETTE_COLLI, NEON, AGES, DEPTHS, SUMMER, OTHER, RECORDS];

/**
 * The bar with the page under it. The bar's scroll spy lights the section whose
 * heading is in view, so the first section's heading is rendered below the bar
 * (with the page tall enough to scroll), exactly as Browse lays them out.
 */
function Page({ chips, tools }: { chips: IChipModel[]; tools: React.ReactNode }) {
    const first = chips[0];
    return (
        <div style={{ width: 840, minHeight: 1600 }}>
            <JumpBar chips={chips} tools={tools} />
            {first ? (
                <section id={first.id} style={{ marginTop: 320 }}>
                    <LibrarySectionHead chip={first} count={`${first.count} chapters`} />
                </section>
            ) : null}
        </div>
    );
}

/** The desktop bar at the top of the page: Act 0 lit and expanded, every other section collapsed to its mark. */
export const AllSections = () => {
    const state = useToolbarState();
    return <Page chips={ALL} tools={<LibraryToolbarButton state={state} />} />;
};

/** A short Archive (filtered to events): the chips fit, so neither edge fades; the button shows two filters on. */
export const FewSections = () => {
    const state = useToolbarState({ filter: "events", readFilter: "progress" });
    return <Page chips={[SNOW, SETTE_COLLI, NEON, OTHER]} tools={<LibraryToolbarButton state={state} />} />;
};

/** A sort flattens the shelves: the bar names the one list and keeps its search button (one filter on: the order). */
export const Sorted = () => {
    const state = useToolbarState({ sort: "newest" });
    return (
        <div style={{ width: 840 }}>
            <JumpBar chips={[]} tools={<LibraryToolbarButton state={state} />} />
        </div>
    );
};
