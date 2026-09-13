import { FilterToggleButton, Input, RosterFilters } from "frontend";
import { type ReactNode, useState } from "react";

// The profile roster tab's filter sidebar: the shared operator-list chrome
// (`FilterPanel`) around the shared field set (`OperatorFilterFields`), with an
// Ownership row (Owned / Unowned / All) leading the Basic section instead of
// the /operators page's Availability row. It sticks below the profile's tab
// bar (`PROFILE_STICKY_OFFSET_PX`), and above `md` it becomes a left sheet.
//
// `filters` is `ISharedFilters` (the ten array facets); `options` is
// `IFilterOptions`, the distinct values the roster derives from the index plus
// the voices table. Ownership is not a "filter" for the count: the badge and
// "Clear all" only light for the shared facets and the search box.

const OPTIONS = {
    subclasses: ["librator", "lord", "fearless", "centurion", "sword", "protector", "guardian", "fastshot", "bombarder", "corecaster", "splashcaster", "pioneer", "bearer", "tactician", "executor", "bard"],
    nations: ["rhodes", "lungmen", "kazimierz", "columbia", "victoria", "siracusa", "kjerag", "yan", "ursus", "sargon", "laterano"],
    factions: ["penguin", "rhine", "karlan", "abyssal", "lgd", "blacksteel", "pinus", "sui", "elite"],
    races: ["Kuranta", "Feline", "Lupo", "Sarkaz", "Oni", "Liberi", "Cautus", "Elf", "Caprinae", "Vulpo", "Aegir"],
    birthPlaces: ["Kazimierz", "Lungmen", "Columbia", "Victoria", "Higashi", "Laterano", "Siracusa", "Kjerag", "Rim Billiton", "Kazdel", "Ægir"],
    artists: ["Skade", "竜崎いち", "幻象黑兔", "Infukun", "NoriZC", "唯@W", "下野宏铭", "Anmi", "LLC", "板板"],
    voiceActors: ["Ayane Sakura", "Tomokazu Sugita", "Rie Takahashi", "Yui Ishikawa", "Maaya Sakamoto", "Kana Hanazawa", "Yoshimasa Hosoya"],
};

const NONE = {
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

const noop = () => {};

/** The sidebar is `position: sticky; flex: 0 0 280px; align-self: stretch` -
 *  it needs the flex row `RosterTab` puts it in, beside the toolbar + grid
 *  column. A slice of that column stands in here so the collapsed story has
 *  something to collapse against. */
function Stage({ visible, activeCount, onToggle, children }: { visible: boolean; activeCount: number; onToggle: () => void; children: ReactNode }) {
    return (
        <div className="flex w-full items-start gap-2">
            {children}
            <div className="flex min-w-0 flex-1 flex-col gap-3">
                <div className="flex w-full items-center gap-3">
                    <FilterToggleButton visible={visible} onToggle={onToggle} activeCount={activeCount} />
                    <Input className="w-64" placeholder="Search operators..." readOnly value="" />
                </div>
                <div className="grid grid-cols-2 gap-3">
                    {["Młynar", "Skadi", "Lappland", "Texas the Omertosa"].map((name) => (
                        <div key={name} className="rounded-lg border border-border bg-card px-4 py-6 text-center text-muted-foreground text-sm">
                            {name}
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
}

// Resting: Owned selected, nothing else on, so there is no "Clear all".
export const Sidebar = () => (
    <Stage visible activeCount={0} onToggle={noop}>
        <RosterFilters filters={NONE} options={OPTIONS} onChange={noop} ownership="owned" onOwnershipChange={noop} onClearAll={noop} hasActiveFilters={false} collapsed={false} onToggle={noop} activeFilterCount={0} />
    </Stage>
);

// Looking for gaps in the roster: Unowned, two classes, 6★ and 5★ only, and
// one advanced facet - the Advanced header counts it and "Clear all" appears.
export const UnownedWithFilters = () => (
    <Stage visible activeCount={5} onToggle={noop}>
        <RosterFilters filters={{ ...NONE, classes: ["WARRIOR", "SNIPER"], rarities: ["TIER_6", "TIER_5"], nations: ["kazimierz"] }} options={OPTIONS} onChange={noop} ownership="unowned" onOwnershipChange={noop} onClearAll={noop} hasActiveFilters collapsed={false} onToggle={noop} activeFilterCount={5} />
    </Stage>
);

// Whole collection with the advanced facets doing the work: a race, a birth
// place and a voice actor, no class or rarity narrowing.
export const AllOperatorsAdvanced = () => (
    <Stage visible activeCount={3} onToggle={noop}>
        <RosterFilters filters={{ ...NONE, races: ["Lupo"], birthPlaces: ["Siracusa"], voiceActors: ["Ayane Sakura"] }} options={OPTIONS} onChange={noop} ownership="all" onOwnershipChange={noop} onClearAll={noop} hasActiveFilters collapsed={false} onToggle={noop} activeFilterCount={3} />
    </Stage>
);

// Hidden by the toolbar toggle: the sidebar collapses to zero width and the
// grid takes the row. Live - the toggle button reopens it.
export const Collapsed = () => {
    const [visible, setVisible] = useState(false);
    const toggle = () => setVisible((v) => !v);
    return (
        <Stage visible={visible} activeCount={2} onToggle={toggle}>
            <RosterFilters filters={{ ...NONE, classes: ["CASTER"], rarities: ["TIER_6"] }} options={OPTIONS} onChange={noop} ownership="owned" onOwnershipChange={noop} onClearAll={noop} hasActiveFilters collapsed={!visible} onToggle={toggle} activeFilterCount={2} />
        </Stage>
    );
};
