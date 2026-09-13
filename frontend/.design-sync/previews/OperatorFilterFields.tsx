import { OperatorFilterFields, TagRow } from "frontend";
import { type ReactNode, useEffect, useRef } from "react";

const noop = () => {};

// `IFilterOptions` - the distinct values `buildFilterOptions` derives from the
// 438-row operator index. Archetype/nation/faction ids are the game's own
// (`librator`, `kazimierz`, `penguin`); the dropdowns format them.
const OPTIONS = {
    subclasses: ["librator", "lord", "fearless", "centurion", "sword", "protector", "guardian", "fastshot", "bombarder", "corecaster", "splashcaster", "pioneer", "bearer", "tactician"],
    nations: ["rhodes", "lungmen", "kazimierz", "columbia", "victoria", "siracusa", "kjerag", "yan", "ursus", "sargon"],
    factions: ["penguin", "rhine", "karlan", "abyssal", "lgd", "blacksteel", "pinus", "sui"],
    races: ["Kuranta", "Feline", "Lupo", "Sarkaz", "Oni", "Liberi", "Cautus", "Elf", "Caprinae", "Vulpo"],
    birthPlaces: ["Kazimierz", "Lungmen", "Columbia", "Victoria", "Higashi", "Laterano", "Siracusa", "Kjerag", "Rim Billiton", "Kazdel"],
    artists: ["Skade", "竜崎いち", "幻象黑兔", "Infukun", "NoriZC", "唯@W", "下野宏铭", "Anmi", "LLC", "板板"],
    voiceActors: ["Ayane Sakura", "Tomokazu Sugita", "Rie Takahashi", "Yui Ishikawa", "Maaya Sakamoto", "Kana Hanazawa"],
};

// `EMPTY_SHARED_FILTERS`
const EMPTY = {
    classes: [],
    subclasses: [],
    rarities: [],
    genders: [],
    nations: [],
    factions: [],
    races: [],
    birthPlaces: [],
    artists: [],
    voiceActors: [],
};

const NARROWED = {
    ...EMPTY,
    classes: ["WARRIOR", "SNIPER"],
    rarities: ["TIER_6", "TIER_5"],
    subclasses: ["librator", "bombarder"],
    genders: ["Female"],
    nations: ["kazimierz"],
    factions: ["penguin"],
};

const AVAILABILITY = [
    { value: "global", label: "Global" },
    { value: "upcoming", label: "Upcoming (CN)" },
] as const;

const NOTES = [
    { value: "any", label: "Any" },
    { value: "yes", label: "Has notes" },
    { value: "no", label: "No notes" },
] as const;

const OWNERSHIP = [
    { value: "owned", label: "Owned" },
    { value: "unowned", label: "Unowned" },
    { value: "all", label: "All" },
] as const;

/** The fields normally sit inside `FilterPanel`'s 280px sidebar card; this
 *  reproduces that inner surface so the container-query rarity grid lays out
 *  the way it does on the page. */
const Sidebar = ({ children }: { children: ReactNode }) => <div className="w-70 rounded-xl border border-border bg-card px-4 py-4.5">{children}</div>;

/** "Advanced" is open by default and owns its state, so the collapsed section
 *  is reached by clicking its header two frames after mount. */
const CollapseAdvanced = ({ children }: { children: ReactNode }) => {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        let inner = 0;
        const outer = requestAnimationFrame(() => {
            inner = requestAnimationFrame(() => (ref.current?.querySelector("button[aria-expanded]") as HTMLElement | null)?.click());
        });
        return () => {
            cancelAnimationFrame(outer);
            cancelAnimationFrame(inner);
        };
    }, []);
    return <div ref={ref}>{children}</div>;
};

/** Nothing selected - Basic (class, rarity) and the open Advanced section. */
export const Empty = () => (
    <Sidebar>
        <OperatorFilterFields filters={EMPTY} options={OPTIONS} onChange={noop} />
    </Sidebar>
);

/** The /operators composition: an Availability row leads Basic, a Notes row trails it. */
export const OperatorsPage = () => (
    <Sidebar>
        <OperatorFilterFields filters={NARROWED} options={OPTIONS} onChange={noop} basicLeading={<TagRow label="Availability" options={AVAILABILITY} value="global" onChange={noop} />} basicTrailing={<TagRow label="Notes" options={NOTES} value="yes" onChange={noop} />} />
    </Sidebar>
);

/** The profile roster composition: only an Ownership row leads Basic. */
export const RosterTab = () => (
    <Sidebar>
        <OperatorFilterFields filters={{ ...EMPTY, rarities: ["TIER_6"], classes: ["CASTER"] }} options={OPTIONS} onChange={noop} basicLeading={<TagRow label="Ownership" options={OWNERSHIP} value="unowned" onChange={noop} />} />
    </Sidebar>
);

/** Advanced folded away with five selections inside it - the count pill next to the label. */
export const AdvancedCollapsed = () => (
    <Sidebar>
        <CollapseAdvanced>
            <OperatorFilterFields filters={NARROWED} options={OPTIONS} onChange={noop} basicLeading={<TagRow label="Availability" options={AVAILABILITY} value="global" onChange={noop} />} basicTrailing={<TagRow label="Notes" options={NOTES} value="any" onChange={noop} />} />
        </CollapseAdvanced>
    </Sidebar>
);
