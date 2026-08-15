import { CommitItem } from "frontend";

// One node on the changelog timeline. CommitItem renders an <li>, so every story
// composes it inside the <ul> the page wraps each day-group in.
const ANCHOR = Date.UTC(2024, 4, 15, 12, 0, 0);
const DAY_MS = 86_400_000;
const iso = (hoursAgo: number) => new Date(ANCHOR - hoursAgo * 3_600_000).toISOString();
const days = (n: number) => new Date(ANCHOR - n * DAY_MS).toISOString();

const ELTIK = {
    name: "Eltik",
    login: "Eltik",
    avatarUrl: "https://avatars.githubusercontent.com/u/76538547?v=4",
    profileUrl: "https://github.com/Eltik",
};

const BOT = { name: "renovate[bot]", login: null, avatarUrl: null, profileUrl: null };

const commit = (over: Record<string, unknown>) => ({
    sha: "a03a012c2ef78ed2fa8823a6785a7c963b2959e8",
    shortSha: "a03a012",
    title: "",
    body: "",
    type: "other",
    scope: null,
    breaking: false,
    url: "https://github.com/Eltik/myrtle/commit/a03a012c",
    date: iso(5),
    author: ELTIK,
    verified: true,
    ...over,
});

const Timeline = ({ children }: { children: React.ReactNode }) => <ul className="m-0 w-full max-w-2xl list-none p-0">{children}</ul>;

export const FeatureCommit = () => (
    <Timeline>
        <CommitItem
            commit={commit({
                shortSha: "fe7ff57",
                type: "feature",
                scope: "dynchar",
                title: "ninth reference — Whislash the Decadenza, from an unscored capture",
                body: "Adds the entrance capture as an opt-in ninth reference so regressions on perspective-camera skins surface before release.",
                date: iso(3),
            })}
        />
    </Timeline>
);

export const BreakingChange = () => (
    <Timeline>
        <CommitItem
            commit={commit({
                shortSha: "e48da79",
                type: "feature",
                scope: "base",
                breaking: true,
                title: "complete the clause-engine parity roadmap; delete perception",
                body: "perception.rs is gone — ledger.rs is now the only scorer. Anything calling scoreRoom() directly must move to a ClauseKind.",
                date: iso(9),
            })}
        />
    </Timeline>
);

export const LongBodyCollapsed = () => (
    <Timeline>
        <CommitItem
            commit={commit({
                shortSha: "a03a012",
                type: "fix",
                scope: "dynchar",
                title: "the entrance POST-PROCESS volume was never applied",
                body: "A whole subsystem was missing: the entrance scene ships a post-process volume that the exporter read but the renderer never bound, so bloom, colour grading and vignette were all inert on every entrance skin. Wiring it through moves whitw2 from 77.490 to 73.264 and leaves the other eight references bit-identical, which is what you want from a fix that only touches a path one skin exercises.",
                date: days(2),
            })}
        />
    </Timeline>
);

export const TimelineGroup = () => (
    <Timeline>
        <CommitItem commit={commit({ shortSha: "5b11924", type: "perf", scope: "dynchar", title: "entrance parity 14.227 -> 12.716 across four fixes", body: "", date: iso(6) })} />
        <CommitItem commit={commit({ shortSha: "def4c03", type: "fix", scope: "dynchar", title: "measureVisibleBounds clobbered the spine animation rate", body: "The bounds pass advanced the skeleton without restoring its timescale.", date: iso(14) })} />
        <CommitItem commit={commit({ shortSha: "67f699a", type: "other", scope: null, title: "dps: add Angelina the Mellow Wish operator formulas", body: "", date: iso(20) })} />
        <CommitItem commit={commit({ shortSha: "9ed026e", type: "chore", scope: null, title: "clippy pedantic/nursery + biome sweep (discord, unpacker, downloader, frontend)", body: "", date: days(1), author: BOT })} />
    </Timeline>
);
