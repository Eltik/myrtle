import type React from "react";
import { Button, LibrarySectionHead } from "frontend";
import type { IChipModel } from "../../src/components/story/library/impl/chapters";

// LibrarySectionHead (the source's `SectionHead`) is the heading over one
// Archives shelf: the section's mark, its NAME as the heading, and a muted
// line reading "<kind> · <chapter range> · <count>". It sits over a thin rule
// and may carry an action at its right end (the records section's show/hide toggle).

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

const Stage = ({ children }: { children: React.ReactNode }) => <div style={{ width: 760 }}>{children}</div>;

/** A mainline arc: the act banner, "Main story · Ch. 4-8 · 6 chapters". */
export const MainlineArc = () => (
    <Stage>
        <LibrarySectionHead chip={ARC_2} count="6 chapters" />
    </Stage>
);

/** A themed shelf that holds mainline chapters without being a run of them. */
export const ThemedShelf = () => (
    <Stage>
        <LibrarySectionHead chip={THE_ARK} count="9 chapters" />
    </Stage>
);

/** An events-only shelf. */
export const EventShelf = () => (
    <Stage>
        <LibrarySectionHead chip={SNOW} count="4 chapters" />
    </Stage>
);

/** The operator records section, with its show/hide toggle at the right. */
export const RecordsWithAction = () => (
    <Stage>
        <LibrarySectionHead
            chip={RECORDS}
            count="319 operators"
            action={
                <Button variant="ghost" size="sm">
                    Show
                </Button>
            }
        />
    </Stage>
);
