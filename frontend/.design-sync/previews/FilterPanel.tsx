import { FilterPanel, FilterToggleButton, OperatorFilterFields, TagRow } from "frontend";
import { Search } from "lucide-react";
import type { ReactNode } from "react";

const noop = () => {};

// One sidebar/sheet chrome for every operator-list surface. From md up it is a
// sticky 280px <aside> with a "Filters" header; below md the same children mount
// in a left `Sheet` (not reachable at the 900px capture width). `collapsed`
// animates the aside to zero width and marks it inert.

const OPTIONS = {
    subclasses: ["librator", "lord", "fearless", "centurion", "sword", "protector", "guardian", "fastshot", "bombarder", "corecaster", "splashcaster", "pioneer", "bearer", "tactician"],
    nations: ["rhodes", "lungmen", "kazimierz", "columbia", "victoria", "siracusa", "kjerag", "yan", "ursus", "sargon"],
    factions: ["penguin", "rhine", "karlan", "abyssal", "lgd", "blacksteel", "pinus", "sui"],
    races: ["Kuranta", "Feline", "Lupo", "Sarkaz", "Oni", "Liberi", "Cautus", "Elf", "Caprinae", "Vulpo"],
    birthPlaces: ["Kazimierz", "Lungmen", "Columbia", "Victoria", "Higashi", "Laterano", "Siracusa", "Kjerag", "Rim Billiton", "Kazdel"],
    artists: ["Skade", "竜崎いち", "幻象黑兔", "Infukun", "NoriZC", "唯@W", "下野宏铭", "Anmi", "LLC", "板板"],
    voiceActors: ["Ayane Sakura", "Tomokazu Sugita", "Rie Takahashi", "Yui Ishikawa", "Maaya Sakamoto", "Kana Hanazawa"],
};

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

const NARROWED = { ...EMPTY, classes: ["WARRIOR", "SNIPER"], rarities: ["TIER_6", "TIER_5"], subclasses: ["librator"], nations: ["kazimierz"] };

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

/** The sidebar is `position: sticky; align-self: stretch` - it needs the flex
 *  row `Operators.tsx` puts it in, beside the results column. */
const Page = ({ children }: { children: ReactNode }) => <div className="flex w-full items-start">{children}</div>;

/** The results column's toolbar: the toggle that pairs with the panel, then the search field. */
const Toolbar = ({ visible, activeCount }: { visible: boolean; activeCount: number }) => (
    <main className="flex min-w-0 flex-1 flex-col gap-3.5" aria-label="Operator results">
        <div className="flex flex-wrap items-center gap-2.5">
            <FilterToggleButton visible={visible} onToggle={noop} activeCount={activeCount} />
            <div className="flex h-10 min-w-60 flex-1 items-center gap-2 rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_60%,transparent)] px-3 sm:max-w-115 [&>svg]:shrink-0 [&>svg]:text-muted-foreground">
                <Search className="h-3.75 w-3.75" aria-hidden="true" />
                <input type="text" defaultValue="" placeholder="Search operators..." aria-label="Search operators" className="min-w-0 flex-1 appearance-none border-0 bg-transparent p-0 font-sans text-foreground text-sm leading-none outline-none placeholder:text-muted-foreground" />
            </div>
        </div>
        <div className="rounded-xl border border-border border-dashed bg-card/50 py-16 text-center">
            <p className="font-sans text-muted-foreground text-sm">{activeCount > 0 ? `${activeCount} filters narrow the list to 74 operators.` : "438 operators."}</p>
        </div>
    </main>
);

/** Open, nothing filtered: header only, no "Clear all". */
export const Open = () => (
    <Page>
        <FilterPanel hasActiveFilters={false} onClearAll={noop} ariaLabel="Operator filters" activeFilterCount={0} onToggle={noop}>
            <OperatorFilterFields filters={EMPTY} options={OPTIONS} onChange={noop} basicLeading={<TagRow label="Availability" options={AVAILABILITY} value="global" onChange={noop} />} basicTrailing={<TagRow label="Notes" options={NOTES} value="any" onChange={noop} />} />
        </FilterPanel>
        <Toolbar visible activeCount={0} />
    </Page>
);

/** Six filters applied: "Clear all" appears in the header. */
export const WithActiveFilters = () => (
    <Page>
        <FilterPanel hasActiveFilters onClearAll={noop} ariaLabel="Operator filters" activeFilterCount={6} onToggle={noop}>
            <OperatorFilterFields filters={NARROWED} options={OPTIONS} onChange={noop} basicLeading={<TagRow label="Availability" options={AVAILABILITY} value="global" onChange={noop} />} basicTrailing={<TagRow label="Notes" options={NOTES} value="any" onChange={noop} />} />
        </FilterPanel>
        <Toolbar visible activeCount={6} />
    </Page>
);

/** Collapsed: the aside shrinks to nothing and the toolbar toggle carries the count badge. */
export const Collapsed = () => (
    <Page>
        <FilterPanel collapsed hasActiveFilters onClearAll={noop} ariaLabel="Operator filters" activeFilterCount={6} onToggle={noop}>
            <OperatorFilterFields filters={NARROWED} options={OPTIONS} onChange={noop} basicLeading={<TagRow label="Availability" options={AVAILABILITY} value="global" onChange={noop} />} basicTrailing={<TagRow label="Notes" options={NOTES} value="any" onChange={noop} />} />
        </FilterPanel>
        <Toolbar visible={false} activeCount={6} />
    </Page>
);

/** The profile roster's instance: an Ownership row leads, and `stickyOffset`
 *  lifts the sticky top below the tab bar (64px header + 44px tabs = 108). */
export const RosterTab = () => (
    <Page>
        <FilterPanel hasActiveFilters onClearAll={noop} ariaLabel="Roster filters" activeFilterCount={2} onToggle={noop} stickyOffset={108}>
            <OperatorFilterFields filters={{ ...EMPTY, rarities: ["TIER_6"], classes: ["MEDIC"] }} options={OPTIONS} onChange={noop} basicLeading={<TagRow label="Ownership" options={OWNERSHIP} value="owned" onChange={noop} />} />
        </FilterPanel>
        <Toolbar visible activeCount={2} />
    </Page>
);
