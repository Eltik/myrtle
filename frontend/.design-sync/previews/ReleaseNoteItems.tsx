import { ReleaseNoteItems } from "frontend";

// The secondary-changes list under a release note's lead, in the dialog and in
// the archive. Each item is a coloured dot plus a "New / Improved / Fixed"
// label, never a filled badge: three coloured pills in a short list read as a
// rainbow. `fixed` is deliberately the quietest. Items are
// `{ kind: "new" | "improved" | "fixed", textKey }`.

// The "Player search ranks by score" entry (2026-09-25): all three kinds. Items
// carry message keys (`entries.messages.ts`, namespace `changelog`), so only
// real shipped entries resolve to text.
const SEARCH_RANKING_ITEMS = [
    { kind: "new", textKey: "note.2026-09-25.item.1" },
    { kind: "new", textKey: "note.2026-09-25.item.2" },
    { kind: "improved", textKey: "note.2026-09-25.item.3" },
    { kind: "improved", textKey: "note.2026-09-25.item.4" },
    { kind: "fixed", textKey: "note.2026-09-25.item.5" },
    { kind: "fixed", textKey: "note.2026-09-25.item.6" },
    { kind: "fixed", textKey: "note.2026-09-25.item.7" },
] as const;

export const MixedKinds = () => (
    <div className="max-w-md">
        <ReleaseNoteItems items={[...SEARCH_RANKING_ITEMS]} />
    </div>
);

// As the dialog frames it: under the "Also in this update" eyebrow, inside a
// bordered panel.
export const InDialogPanel = () => (
    <div className="max-w-md rounded-xl border border-border bg-card p-4">
        <p className="mb-3 font-medium text-[0.69rem] text-muted-foreground uppercase tracking-[0.18em]">Also in this update</p>
        <ReleaseNoteItems items={[...SEARCH_RANKING_ITEMS].slice(0, 4)} />
    </div>
);

// A maintenance entry: nothing new, every line a fix, so the whole list sits in
// the quiet grey.
export const FixesOnly = () => (
    <div className="max-w-md">
        <ReleaseNoteItems
            items={[
                { kind: "fixed", textKey: "note.2026-10-07.item.1" },
                { kind: "fixed", textKey: "note.2026-10-07.item.2" },
                { kind: "fixed", textKey: "note.2026-10-07.item.3" },
            ]}
        />
    </div>
);

// A single long line, to show the dot stays on the first line while the text
// wraps under the label.
export const LongItem = () => (
    <div className="max-w-xs">
        <ReleaseNoteItems items={[{ kind: "fixed", textKey: "note.2026-09-21.item.12" }]} />
    </div>
);
