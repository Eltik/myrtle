import { ReleaseNoteItems } from "frontend";

// The secondary-changes list under a release note's lead, in the dialog and in
// the archive. Each item is a coloured dot plus a "New / Improved / Fixed"
// label, never a filled badge: three coloured pills in a short list read as a
// rainbow. `fixed` is deliberately the quietest. Items are
// `{ kind: "new" | "improved" | "fixed", text }`.

// The list shipped with the "Build statistics" entry, all three kinds.
const BUILD_STATS_ITEMS = [
    { kind: "new", text: "Added changelog notifications." },
    { kind: "new", text: "Added build statistics." },
    { kind: "new", text: "Display breakpoints and percentages for mastery/module levels." },
    { kind: "improved", text: "Operator skills/modules default to what is most used." },
    { kind: "improved", text: "Operator grades were computed from the wrong baseline. Scores across the roster have shifted." },
    { kind: "fixed", text: "Profile rosters no longer play E2 dynamic art over an operator who has not reached E2." },
    { kind: "fixed", text: "Check-in dates and Reclamation Algorithm scoring." },
] as const;

export const MixedKinds = () => (
    <div className="max-w-md">
        <ReleaseNoteItems items={[...BUILD_STATS_ITEMS]} />
    </div>
);

// As the dialog frames it: under the "Also in this update" eyebrow, inside a
// bordered panel.
export const InDialogPanel = () => (
    <div className="max-w-md rounded-xl border border-border bg-card p-4">
        <p className="mb-3 font-medium text-[0.69rem] text-muted-foreground uppercase tracking-[0.18em]">Also in this update</p>
        <ReleaseNoteItems items={[...BUILD_STATS_ITEMS].slice(0, 4)} />
    </div>
);

// A maintenance entry: nothing new, every line a fix, so the whole list sits in
// the quiet grey.
export const FixesOnly = () => (
    <div className="max-w-md">
        <ReleaseNoteItems
            items={[
                { kind: "fixed", text: "Base planner no longer schedules an empty trading post on the night shift." },
                { kind: "fixed", text: "Skin viewer keeps the chosen outfit when switching between E1 and E2 art." },
                { kind: "fixed", text: "Leaderboard pagination skipped page 2 after a search." },
            ]}
        />
    </div>
);

// A single long line, to show the dot stays on the first line while the text
// wraps under the label.
export const LongItem = () => (
    <div className="max-w-xs">
        <ReleaseNoteItems items={[{ kind: "improved", text: "The base optimizer now prices every one of the 691 room buffs it knows about, so the shift rotation it recommends matches what the in-game assistant would suggest for the same roster." }]} />
    </div>
);
