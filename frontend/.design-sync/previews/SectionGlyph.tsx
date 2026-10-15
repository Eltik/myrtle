import { SectionGlyph } from "frontend";
import type { IChipModel } from "../../src/components/story/library/impl/chapters";

// SectionGlyph is a section's mark: the arc's 184x52 act banner (sized by
// height), a themed shelf's 108x108 logo (a square slot), or, where the wire
// sends no art, a lucide glyph the section id hashes to. The game art is
// monochrome and is inked to the theme; on a lit (active) jump chip it is
// inked white over the primary fill. It leads every section heading and every
// jump-bar chip.

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

/** The three mark shapes at heading size: an act banner, a shelf logo, a fallback glyph. */
export const HeadingSize = () => (
    <div className="flex items-center gap-8 text-muted-foreground">
        <SectionGlyph chip={ARC_2} place="head" />
        <SectionGlyph chip={THE_ARK} place="head" />
        <SectionGlyph chip={OTHER} place="head" />
        <SectionGlyph chip={RECORDS} place="head" />
    </div>
);

/** At chip size, resting and lit: a lit chip inks the art white over the primary fill. */
export const ChipSizeLit = () => (
    <div className="flex flex-col gap-3">
        {[false, true].map((lit) => (
            <div key={String(lit)} className="flex items-center gap-2">
                {[ARC_1, SNOW, WILDFIRE, OTHER].map((chip) => (
                    <span key={chip.id} className={lit ? "flex items-center rounded-[10px] border border-primary bg-primary px-2 py-1.5 text-primary-foreground" : "flex items-center rounded-[10px] border border-border px-2 py-1.5 text-muted-foreground"}>
                        <SectionGlyph chip={chip} place="chip" lit={lit} alone={chip.iconLogo === true} />
                    </span>
                ))}
            </div>
        ))}
    </div>
);
