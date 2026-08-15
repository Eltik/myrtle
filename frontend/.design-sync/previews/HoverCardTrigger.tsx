import { HoverCard, HoverCardContent, HoverCardTrigger } from "frontend";

const AVATAR = (id: string) => `https://api.myrtle.moe/api/avatar/${id}`;

const summary = (name: string, meta: string, blurb: string) => (
    <div className="flex flex-col gap-2">
        <span className="font-semibold text-sm">{name}</span>
        <span className="text-muted-foreground text-xs">{meta}</span>
        <p className="text-foreground text-xs leading-snug">{blurb}</p>
    </div>
);

/** `render` swaps in a link — how the operator list wires a card to /operators/$id. */
export const AsOperatorLink = () => (
    <div className="flex h-96 w-full items-start justify-center pt-2">
        <HoverCard open>
            <HoverCardTrigger
                render={
                    // biome-ignore lint/a11y/useValidAnchor: static preview, no navigation
                    <a className="group flex w-32 cursor-pointer flex-col rounded bg-card pt-1 pr-2 pb-1 pl-1.5" href="#operators">
                        <span className="ml-px truncate text-foreground text-xs leading-tight">Eyjafjalla</span>
                        <span className="relative box-content block aspect-square h-20 overflow-hidden" style={{ borderBottom: "4px solid #f7a452" }}>
                            <img alt="Eyjafjalla" className="relative h-full w-full object-contain" src={AVATAR("char_180_amgoat")} />
                        </span>
                        <span className="mt-0.5 block truncate text-center text-muted-foreground text-xs leading-tight">Core Caster</span>
                    </a>
                }
            />
            <HoverCardContent>{summary("Eyjafjalla", "6★ Caster · Core Caster · Rhodes Island", "Highest single-target arts damage in the game at S3M3. Ranged, 2-block-free, heavy DP cost.")}</HoverCardContent>
        </HoverCard>
    </div>
);

/** The default trigger element (a `<button>`) styled as a compact roster chip. */
export const DefaultElement = () => (
    <div className="flex h-96 w-full items-start justify-center pt-2">
        <HoverCard open>
            <HoverCardTrigger className="flex cursor-pointer items-center gap-2 rounded-md border bg-card px-3 py-2">
                <img alt="" className="size-8 rounded-md object-cover" src={AVATAR("char_1028_texas2")} />
                <span className="font-medium text-foreground text-sm">Texas the Omertosa</span>
            </HoverCardTrigger>
            <HoverCardContent>{summary("Texas the Omertosa", "6★ Specialist · Ambusher · Lungmen", "Invisible while undeployed. S2 chains arts damage across every enemy in a lane.")}</HoverCardContent>
        </HoverCard>
    </div>
);

/** Trigger on a stat value inside a profile card. */
export const OnStatValue = () => (
    <div className="flex h-96 w-full items-start justify-center pt-2">
        <div className="w-64 rounded-lg border bg-card p-4">
            <div className="font-medium text-muted-foreground text-xs uppercase tracking-wide">Sanity spent</div>
            <HoverCard open>
                <HoverCardTrigger className="mt-1 cursor-pointer font-mono font-semibold text-2xl text-foreground tabular-nums underline">4,812</HoverCardTrigger>
                <HoverCardContent className="w-56">
                    <div className="flex flex-col gap-1.5 text-xs">
                        <div className="flex justify-between gap-2">
                            <span className="text-muted-foreground">Chapter 8</span>
                            <span className="font-mono text-foreground tabular-nums">2,160</span>
                        </div>
                        <div className="flex justify-between gap-2">
                            <span className="text-muted-foreground">Annihilation</span>
                            <span className="font-mono text-foreground tabular-nums">1,485</span>
                        </div>
                        <div className="flex justify-between gap-2">
                            <span className="text-muted-foreground">Supply stages</span>
                            <span className="font-mono text-foreground tabular-nums">1,167</span>
                        </div>
                    </div>
                </HoverCardContent>
            </HoverCard>
            <div className="mt-1 text-muted-foreground text-xs">Last 30 days</div>
        </div>
    </div>
);
