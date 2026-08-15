import { Badge, HoverCard, HoverCardContent, HoverCardTrigger } from "frontend";

const AVATAR = (id: string) => `https://api.myrtle.moe/api/avatar/${id}`;

const RESULTS = [
    { id: "char_4064_mlynar", name: "Młynar", stars: "★★★★★★", cls: "Guard", tags: ["Top Operator", "Melee", "DPS", "Survival"] },
    { id: "char_102_texas", name: "Texas", stars: "★★★★★", cls: "Vanguard", tags: ["Senior Operator", "Melee", "DP-Recovery"] },
    { id: "char_180_amgoat", name: "Eyjafjalla", stars: "★★★★★★", cls: "Caster", tags: ["Top Operator", "Ranged", "DPS", "AoE"] },
];

// A bare element, not a component: Base UI's `render` clones what it is given,
// so a wrapper component would swallow the trigger's props and ref.
const resultRow = (result: (typeof RESULTS)[number]) => (
    <button className="flex w-72 cursor-pointer items-center gap-2 rounded-md border bg-card px-2 py-2 text-left" type="button">
        <span aria-hidden="true" className="inline-flex size-8 shrink-0 items-center justify-center overflow-hidden rounded-md bg-muted">
            <img alt="" className="block h-full w-full object-cover" src={AVATAR(result.id)} />
        </span>
        <span className="min-w-0 flex-1">
            <span className="block truncate font-medium text-foreground text-sm leading-tight">{result.name}</span>
            <span className="flex items-center gap-1.5 text-muted-foreground text-xs">
                <span className="font-mono" style={{ color: "#f7a452" }}>
                    {result.stars}
                </span>
                <span aria-hidden="true">·</span>
                <span className="truncate">{result.cls}</span>
            </span>
        </span>
    </button>
);

const tagPanel = (tags: string[]) => (
    <div className="flex flex-col gap-2">
        <div className="font-medium text-muted-foreground text-xs">Tags</div>
        <div className="flex flex-wrap gap-1">
            {tags.map((tag) => (
                <Badge key={tag} size="default" variant="outline">
                    {tag}
                </Badge>
            ))}
        </div>
    </div>
);

/** Open — the recruitment calculator's tag breakdown, as ResultCard composes it. */
export const RecruitmentTags = () => (
    <div className="flex h-96 w-full items-start justify-center pt-2">
        <HoverCard open>
            <HoverCardTrigger render={resultRow(RESULTS[0])} />
            <HoverCardContent className="w-max max-w-72 p-3">{tagPanel(RESULTS[0].tags)}</HoverCardContent>
        </HoverCard>
    </div>
);

/** Closed — the guaranteed-6★ result list before the pointer reaches a row. */
export const Closed = () => (
    <ul className="flex w-full flex-col gap-1.5">
        {RESULTS.map((result) => (
            <li key={result.id}>
                <HoverCard>
                    <HoverCardTrigger render={resultRow(result)} />
                    <HoverCardContent className="w-max max-w-72 p-3">{tagPanel(result.tags)}</HoverCardContent>
                </HoverCard>
            </li>
        ))}
    </ul>
);
