import { CampIcon, FilterDropdown, SubProfessionIcon, TeamIcon } from "frontend";
import { type ReactNode, useEffect, useRef } from "react";

const noop = () => {};

const ARCHETYPE_LABELS: Record<string, string> = {
    librator: "Liberator Guard",
    lord: "Lord Guard",
    fearless: "Dreadnought Guard",
    centurion: "Centurion Guard",
    sword: "Swordmaster Guard",
    protector: "Protector Defender",
    guardian: "Guardian Defender",
    artsprotector: "Arts Protector Defender",
    fastshot: "Marksman Sniper",
    bombarder: "Flinger Sniper",
    aoesniper: "Artilleryman Sniper",
    corecaster: "Core Caster",
    splashcaster: "Splash Caster",
    funnel: "Mech-Accord Caster",
    pioneer: "Pioneer Vanguard",
    bearer: "Standard Bearer Vanguard",
    tactician: "Tactician Vanguard",
};

const ARCHETYPE_CLASS: Record<string, string> = {
    librator: "WARRIOR",
    lord: "WARRIOR",
    fearless: "WARRIOR",
    centurion: "WARRIOR",
    sword: "WARRIOR",
    protector: "TANK",
    guardian: "TANK",
    artsprotector: "TANK",
    fastshot: "SNIPER",
    bombarder: "SNIPER",
    aoesniper: "SNIPER",
    corecaster: "CASTER",
    splashcaster: "CASTER",
    funnel: "CASTER",
    pioneer: "PIONEER",
    bearer: "PIONEER",
    tactician: "PIONEER",
};

const CLASS_LABELS: Record<string, string> = {
    PIONEER: "Vanguard",
    WARRIOR: "Guard",
    TANK: "Defender",
    SNIPER: "Sniper",
    CASTER: "Caster",
};

const ARCHETYPES = Object.keys(ARCHETYPE_LABELS);

const NATION_LABELS: Record<string, string> = {
    rhodes: "Rhodes Island",
    lungmen: "Lungmen",
    kazimierz: "Kazimierz",
    columbia: "Columbia",
    victoria: "Victoria",
    siracusa: "Siracusa",
    kjerag: "Kjerag",
    yan: "Yan",
    ursus: "Ursus",
    sargon: "Sargon",
};

const FACTION_LABELS: Record<string, string> = {
    penguin: "Penguin Logistics",
    rhine: "Rhine Lab",
    karlan: "Karlan Trade",
    abyssal: "Abyssal Hunters",
    lgd: "L.G.D.",
    blacksteel: "Blacksteel",
    pinus: "Pinus Sylvestris",
    sui: "Yan Sui",
};

const ARTISTS = ["Skade", "竜崎いち", "幻象黑兔", "Infukun", "NoriZC", "唯@W", "下野宏铭", "Anmi", "LLC", "板板"];

/** Base UI wires the combobox trigger after first paint, so the click that opens
 *  the popup has to wait two frames. */
const AutoOpen = ({ children }: { children: ReactNode }) => {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        let inner = 0;
        const outer = requestAnimationFrame(() => {
            inner = requestAnimationFrame(() => {
                const trigger = ref.current?.querySelector("[data-slot=combobox-trigger]") as HTMLElement | null;
                trigger?.click();
            });
        });
        return () => {
            cancelAnimationFrame(outer);
            cancelAnimationFrame(inner);
        };
    }, []);
    return (
        <div className="flex min-h-[520px] w-full items-start justify-start" ref={ref}>
            <div className="w-72">{children}</div>
        </div>
    );
};

export const GroupedArchetypes = () => (
    <AutoOpen>
        <FilterDropdown
            label="Archetype"
            placeholder="Select archetype"
            options={ARCHETYPES}
            selected={[]}
            onChange={noop}
            formatOption={(v) => ARCHETYPE_LABELS[v] ?? v}
            groupBy={(v) => ARCHETYPE_CLASS[v] ?? "PIONEER"}
            groupOrder={["PIONEER", "WARRIOR", "TANK", "SNIPER", "CASTER"]}
            formatGroup={(k) => CLASS_LABELS[k] ?? k}
            renderOptionIcon={(v) => <SubProfessionIcon subProfession={v} size={18} />}
        />
    </AutoOpen>
);

export const WithSelectedTags = () => (
    <div className="w-72">
        <FilterDropdown
            label="Faction"
            placeholder="Select faction"
            options={Object.keys(FACTION_LABELS)}
            selected={["penguin", "rhine", "abyssal"]}
            onChange={noop}
            formatOption={(v) => FACTION_LABELS[v] ?? v}
            renderOptionIcon={(v) => <CampIcon groupId={v} size={18} />}
        />
    </div>
);

export const NationOptions = () => (
    <AutoOpen>
        <FilterDropdown label="Nation" placeholder="Select nation" options={Object.keys(NATION_LABELS)} selected={["kazimierz"]} onChange={noop} formatOption={(v) => NATION_LABELS[v] ?? v} renderOptionIcon={(v) => <TeamIcon teamId={v} size={18} />} />
    </AutoOpen>
);

export const PlainOptions = () => (
    <div className="flex w-full flex-col gap-4">
        <div className="w-72">
            <FilterDropdown label="Artist" placeholder="Select artist" options={ARTISTS} selected={["Skade", "Infukun"]} onChange={noop} />
        </div>
        <div className="w-72">
            <FilterDropdown label="Race" placeholder="Select race" options={["Kuranta", "Feline", "Lupo", "Sarkaz", "Oni", "Liberi", "Cautus", "Elf"]} selected={[]} onChange={noop} />
        </div>
    </div>
);
