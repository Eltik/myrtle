import { OperatorFilters } from "frontend";

const noop = () => {};

// `IFilterOptions` — the distinct values `useOperatorFilters` derives from the
// 438-row operator index.
const OPTIONS = {
    subclasses: ["librator", "lord", "fearless", "centurion", "sword", "protector", "guardian", "fastshot", "bombarder", "corecaster", "splashcaster", "pioneer", "bearer", "tactician"],
    nations: ["rhodes", "lungmen", "kazimierz", "columbia", "victoria", "siracusa", "kjerag", "yan", "ursus", "sargon"],
    factions: ["penguin", "rhine", "karlan", "abyssal", "lgd", "blacksteel", "pinus", "sui"],
    races: ["Kuranta", "Feline", "Lupo", "Sarkaz", "Oni", "Liberi", "Cautus", "Elf", "Caprinae", "Vulpo"],
    birthPlaces: ["Kazimierz", "Lungmen", "Columbia", "Victoria", "Higashi", "Laterano", "Siracusa", "Kjerag", "Rim Billiton", "Kazdel"],
    artists: ["Skade", "竜崎いち", "幻象黑兔", "Infukun", "NoriZC", "唯@W", "下野宏铭", "Anmi", "LLC", "板板"],
    voiceActors: ["Ayane Sakura", "Tomokazu Sugita", "Rie Takahashi", "Yui Ishikawa", "Maaya Sakamoto", "Kana Hanazawa"],
};

const BASE = {
    selectedClasses: [],
    selectedSubclasses: [],
    selectedRarities: [],
    selectedGenders: [],
    selectedNations: [],
    selectedFactions: [],
    selectedRaces: [],
    selectedBirthPlaces: [],
    selectedArtists: [],
    selectedVoiceActors: [],
    selectedHasNotes: "any",
    selectedAvailability: "global",
    options: OPTIONS,
    onClassesChange: noop,
    onSubclassesChange: noop,
    onRaritiesChange: noop,
    onGendersChange: noop,
    onNationsChange: noop,
    onFactionsChange: noop,
    onRacesChange: noop,
    onBirthPlacesChange: noop,
    onArtistsChange: noop,
    onVoiceActorsChange: noop,
    onHasNotesChange: noop,
    onAvailabilityChange: noop,
    onClearAll: noop,
    hasActiveFilters: false,
    collapsed: false,
    onToggle: noop,
    activeFilterCount: 0,
};

/** The sidebar is `position: sticky; align-self: stretch` — it needs the flex row
 *  that `Operators.tsx` puts it in. */
const Stage = ({ children }: { children: React.ReactNode }) => <div className="flex w-full items-start">{children}</div>;

export const Sidebar = () => (
    <Stage>
        <OperatorFilters {...BASE} />
    </Stage>
);

export const ActiveFilters = () => (
    <Stage>
        <OperatorFilters {...BASE} selectedClasses={["WARRIOR", "SNIPER"]} selectedRarities={["TIER_6", "TIER_5"]} selectedGenders={["Female"]} selectedSubclasses={["librator", "bombarder"]} selectedNations={["kazimierz"]} selectedHasNotes="yes" hasActiveFilters activeFilterCount={8} />
    </Stage>
);

export const UpcomingAvailability = () => (
    <Stage>
        <OperatorFilters {...BASE} selectedAvailability="upcoming" selectedClasses={["CASTER"]} selectedRarities={["TIER_6"]} hasActiveFilters activeFilterCount={2} />
    </Stage>
);
