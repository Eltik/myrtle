import { CatalogGrid } from "frontend";

// Live /stats snapshot: the Global gamedata revision myrtle currently mirrors.
const GAME_DATA = { operators: 438, skills: 1604, modules: 857, skins: 2063, stages: 3315, zones: 433, enemies: 1590 };
const TIER_LISTS = { total: 80, active: 79, totalVersions: 34, totalPlacements: 6003 };
const ROSTERS = { total: 1515 };

export const Default = () => (
    <section className="flex flex-col gap-4 rounded-[14px] border border-border bg-card p-6">
        <header className="flex flex-col gap-1">
            <span className="mb-2.5 inline-block font-bold text-[0.69rem] text-primary uppercase tracking-[0.22em]">Game catalog</span>
            <h2 className="m-0 font-sans font-semibold text-[22px] text-foreground leading-[1.15] tracking-[-0.02em]">What we know about Terra.</h2>
        </header>
        <CatalogGrid gameData={GAME_DATA} tierLists={TIER_LISTS} rosters={ROSTERS} />
    </section>
);

/** A freshly-deployed instance: gamedata is mirrored, nothing community-side exists yet. */
export const FreshDeployment = () => (
    <section className="flex flex-col gap-4 rounded-[14px] border border-border bg-card p-6">
        <header className="flex flex-col gap-1">
            <span className="mb-2.5 inline-block font-bold text-[0.69rem] text-primary uppercase tracking-[0.22em]">Game catalog</span>
            <h2 className="m-0 font-sans font-semibold text-[22px] text-foreground leading-[1.15] tracking-[-0.02em]">Gamedata mirrored, community empty.</h2>
        </header>
        <CatalogGrid gameData={GAME_DATA} tierLists={{ total: 0, active: 0, totalVersions: 0, totalPlacements: 0 }} rosters={{ total: 0 }} />
    </section>
);
