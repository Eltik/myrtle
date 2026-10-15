import { KindBadge } from "frontend";

// KindBadge names a story's shelf: Main Story, Event, Intermezzo, Vignette or
// Operator Record. It is deliberately ONE neutral pill for every kind (black
// on white in light mode, white on near-black in dark) so the label stays
// legible whatever art or surface it lands on. It appears on the ranking rows
// of Reading Stats, the Reading Order list, the Community list and over the
// Illustrations tab's thumbnails.

/** All five kinds, as they read on a muted panel. */
export const AllKinds = () => (
    <div className="flex flex-wrap items-center gap-2 rounded-lg border border-border bg-muted p-4">
        <KindBadge kind="main" />
        <KindBadge kind="event" />
        <KindBadge kind="intermezzo" />
        <KindBadge kind="vignette" />
        <KindBadge kind="record" />
    </div>
);

const ROWS = [
    { kind: "main", name: "Absolved Will Be the Seekers", words: "97.9K" },
    { kind: "event", name: "Lone Trail", words: "79.9K" },
    { kind: "intermezzo", name: "Under Tides", words: "38.5K" },
    { kind: "vignette", name: "Pinus Sylvestris", words: "20.9K" },
] as const;

/** Leading a ranking row, the way Reading Stats lists the longest chapters. */
export const InARow = () => (
    <ol className="m-0 flex list-none flex-col p-0" style={{ width: 520 }}>
        {ROWS.map((row, at) => (
            <li key={row.name} className="flex items-center gap-2.5 border-border/60 border-b py-1.5 last:border-b-0">
                <span className="w-6 shrink-0 text-right font-mono text-[10px] text-muted-foreground tabular-nums">{at + 1}</span>
                <KindBadge kind={row.kind} className="shrink-0" />
                <span className="min-w-0 flex-1 truncate font-sans text-[12.5px] text-foreground">{row.name}</span>
                <span className="w-16 shrink-0 text-right font-mono text-[10.5px] text-muted-foreground tabular-nums">{row.words}</span>
            </li>
        ))}
    </ol>
);

/** Pinned over a thumbnail, as on the Illustrations tab. */
export const OverArt = () => (
    <div className="relative overflow-hidden rounded-md bg-muted" style={{ width: 260, height: 150 }}>
        <img src="https://api.myrtle.moe/api/assets/textures/spritepack/mixstory_kv_sprites_1/kv_shatterpoint.png" alt="" className="h-full w-full object-cover" />
        <KindBadge kind="main" className="absolute top-1.5 left-1.5 shadow-sm/10" />
    </div>
);
