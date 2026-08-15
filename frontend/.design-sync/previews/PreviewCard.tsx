import { PreviewCard, PreviewCardPopup, PreviewCardTrigger } from "frontend";

const AVATAR = (id: string) => `https://api.myrtle.moe/api/avatar/${id}`;

const RARITY_HEX: Record<number, string> = { 6: "#f7a452", 5: "#f7e79e", 4: "#bcabdb" };

const ROSTER = [
    { id: "char_4064_mlynar", name: "Młynar", rarity: 6, level: 90, elite: 2, cls: "Guard", archetype: "Soloblade", nation: "Kazimierz", complete: 100 },
    { id: "char_263_skadi", name: "Skadi", rarity: 6, level: 80, elite: 2, cls: "Guard", archetype: "Dreadnought", nation: "Rhodes Island", complete: 74 },
    { id: "char_102_texas", name: "Texas", rarity: 5, level: 70, elite: 2, cls: "Vanguard", archetype: "Pioneer", nation: "Columbia", complete: 62 },
];

/** The operator grid tile used across /operators — hover reveals the full stat line. */
function OperatorTile({ op }: { op: (typeof ROSTER)[number] }) {
    const color = RARITY_HEX[op.rarity] ?? "#ffffff";
    return (
        <div className="group relative flex w-32 flex-col rounded bg-card pt-1 pr-2 pb-1 pl-1.5">
            <div className="ml-px flex h-5 flex-col justify-center text-left">
                <span className="truncate text-foreground text-xs leading-tight">{op.name}</span>
            </div>
            <div className="relative box-content aspect-square h-20 overflow-hidden" style={{ borderBottom: `4px solid ${color}` }}>
                <img alt={op.name} className="relative h-full w-full object-contain" src={AVATAR(op.id)} />
            </div>
            <div className="mt-0.5 truncate text-center text-muted-foreground text-xs leading-tight">{op.archetype}</div>
        </div>
    );
}

function OperatorSummary({ op }: { op: (typeof ROSTER)[number] }) {
    const color = RARITY_HEX[op.rarity] ?? "#ffffff";
    return (
        <div className="flex flex-col gap-2.5">
            <div className="flex items-center gap-2.5">
                <img alt={op.name} className="h-10 w-10 rounded-lg object-cover" src={AVATAR(op.id)} style={{ background: `${color}18` }} />
                <div className="min-w-0 flex-1">
                    <div className="truncate font-semibold text-sm leading-tight">{op.name}</div>
                    <div className="font-mono text-muted-foreground text-xs tabular-nums">
                        {op.rarity}★ · E{op.elite} Lv {op.level}
                    </div>
                </div>
            </div>
            <div className="flex items-center gap-1.5 text-muted-foreground text-xs">
                <span className="text-foreground">{op.cls}</span>
                <span aria-hidden="true">·</span>
                <span>{op.archetype}</span>
            </div>
            <div className="text-muted-foreground text-xs">{op.nation}</div>
        </div>
    );
}

/** Open on an operator grid tile — the site's default hover preview. */
export const OperatorHover = () => (
    <div className="flex h-96 w-full items-start justify-center pt-2">
        <PreviewCard open>
            <PreviewCardTrigger className="block">
                <OperatorTile op={ROSTER[0]} />
            </PreviewCardTrigger>
            <PreviewCardPopup>
                <OperatorSummary op={ROSTER[0]} />
            </PreviewCardPopup>
        </PreviewCard>
    </div>
);

/** Open on a roster completion tile, showing what is still missing on that operator. */
export const RosterCompletion = () => (
    <div className="flex h-96 w-full items-start justify-center pt-2">
        <PreviewCard open>
            <PreviewCardTrigger className="block">
                <div className="relative flex w-44 flex-col gap-2.5 rounded-lg border bg-card p-3">
                    <div aria-hidden="true" className="absolute inset-x-0 top-0 h-1 rounded-t-lg" style={{ background: RARITY_HEX[6], opacity: 0.55 }} />
                    <div className="flex items-center gap-2.5 pt-0.5">
                        <img alt="Skadi" className="h-10 w-10 rounded-lg object-cover" src={AVATAR("char_263_skadi")} />
                        <div className="min-w-0 flex-1">
                            <div className="truncate font-semibold text-sm leading-tight">Skadi</div>
                            <div className="font-mono text-muted-foreground text-xs">Lv 80</div>
                        </div>
                    </div>
                    <div className="flex items-center gap-2">
                        <div className="h-1 flex-1 overflow-hidden rounded-full bg-muted">
                            <div className="h-full rounded-full bg-primary" style={{ width: "74%" }} />
                        </div>
                        <span className="font-mono text-muted-foreground text-xs tabular-nums">74%</span>
                    </div>
                </div>
            </PreviewCardTrigger>
            <PreviewCardPopup className="w-56">
                <div className="flex flex-col gap-2.5">
                    <div className="flex items-center justify-between gap-3">
                        <span className="truncate font-semibold text-sm leading-tight">Skadi</span>
                        <span className="shrink-0 font-mono text-muted-foreground text-xs tabular-nums">74%</span>
                    </div>
                    <span className="font-medium text-muted-foreground text-xs uppercase tracking-wide">Remaining</span>
                    <ul className="flex flex-col gap-1">
                        <li className="flex items-center gap-2 text-foreground text-xs">
                            <span aria-hidden="true" className="h-1 w-1 shrink-0 rounded-full bg-primary" />
                            Level 80 → 90
                        </li>
                        <li className="flex items-center gap-2 text-foreground text-xs">
                            <span aria-hidden="true" className="h-1 w-1 shrink-0 rounded-full bg-primary" />
                            Skill 3 mastery M3
                        </li>
                        <li className="flex items-center gap-2 text-foreground text-xs">
                            <span aria-hidden="true" className="h-1 w-1 shrink-0 rounded-full bg-primary" />
                            Module CHA-X unlocked
                        </li>
                    </ul>
                </div>
            </PreviewCardPopup>
        </PreviewCard>
    </div>
);

/** Resting state — a row of triggers before any pointer lands on them. */
export const Resting = () => (
    <div className="flex w-full flex-wrap gap-2">
        {ROSTER.map((op) => (
            <PreviewCard key={op.id}>
                <PreviewCardTrigger className="block">
                    <OperatorTile op={op} />
                </PreviewCardTrigger>
                <PreviewCardPopup>
                    <OperatorSummary op={op} />
                </PreviewCardPopup>
            </PreviewCard>
        ))}
    </div>
);
