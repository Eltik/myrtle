import { ScrollArea } from "frontend";

const MATERIALS = [
    { name: "Bipolar Nanoflake", tier: "T5", have: 11 },
    { name: "D32 Steel", tier: "T5", have: 4 },
    { name: "Polymerization Preparation", tier: "T5", have: 3 },
    { name: "Crystalline Electronic Unit", tier: "T5", have: 0 },
    { name: "Orirock Concentration", tier: "T4", have: 26 },
    { name: "Grindstone Pentahydrate", tier: "T4", have: 9 },
    { name: "White Horse Kohl", tier: "T4", have: 7 },
    { name: "Manganese Trihydrate", tier: "T4", have: 12 },
];

const CHAPTERS = ["Ch. 4 · Beyond Here", "Ch. 5 · Necessary Solution", "Ch. 6 · Partial Necrosis", "Ch. 7 · Preluding Lights", "Ch. 8 · Roaring Flare", "Ch. 9 · Stultifera Navis"];

/** The vertical scrollbar mounted by `ScrollArea` on a depot list that overflows. */
export const VerticalOverflow = () => (
    <div className="h-64 w-80 rounded-lg border bg-card p-2">
        <ScrollArea>
            <div className="flex flex-col divide-y pr-3">
                {MATERIALS.map((mat) => (
                    <div className="flex items-baseline justify-between gap-2 py-2" key={mat.name}>
                        <span className="min-w-0 truncate text-foreground text-sm">{mat.name}</span>
                        <span className="shrink-0 font-mono text-muted-foreground text-xs tabular-nums">
                            {mat.tier} · {mat.have}
                        </span>
                    </div>
                ))}
            </div>
        </ScrollArea>
    </div>
);

/** The horizontal scrollbar, on a chapter strip wider than its container. */
export const HorizontalOverflow = () => (
    <div className="h-20 w-80 rounded-lg border bg-card p-2">
        <ScrollArea>
            <div className="flex w-max gap-2 pb-3">
                {CHAPTERS.map((chapter) => (
                    <span className="whitespace-nowrap rounded-md border bg-background px-3 py-2 text-foreground text-sm" key={chapter}>
                        {chapter}
                    </span>
                ))}
            </div>
        </ScrollArea>
    </div>
);

/** Both axes overflow, so both scrollbars plus the corner are mounted. */
export const BothAxes = () => (
    <div className="h-64 w-80 rounded-lg border bg-card p-2">
        <ScrollArea>
            <table className="w-max border-collapse text-sm">
                <thead>
                    <tr className="text-left text-muted-foreground text-xs">
                        <th className="px-3 py-2 font-medium">Material</th>
                        <th className="px-3 py-2 font-medium">Tier</th>
                        <th className="px-3 py-2 font-medium">Owned</th>
                        <th className="px-3 py-2 font-medium">Best stage</th>
                    </tr>
                </thead>
                <tbody className="divide-y">
                    {MATERIALS.map((mat) => (
                        <tr key={mat.name}>
                            <td className="whitespace-nowrap px-3 py-2 text-foreground">{mat.name}</td>
                            <td className="px-3 py-2 font-mono text-muted-foreground text-xs">{mat.tier}</td>
                            <td className="px-3 py-2 font-mono text-foreground text-xs tabular-nums">{mat.have}</td>
                            <td className="whitespace-nowrap px-3 py-2 font-mono text-muted-foreground text-xs">Synthesis only</td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </ScrollArea>
    </div>
);
