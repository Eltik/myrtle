import { HoverCard, HoverCardContent, HoverCardTrigger } from "frontend";

const AVATAR = (id: string) => `https://api.myrtle.moe/api/avatar/${id}`;

const chip = (id: string, label: string) => (
    <HoverCardTrigger className="flex cursor-pointer items-center gap-2 rounded-md border bg-card px-3 py-2">
        <img alt="" className="size-8 rounded-md object-cover" src={AVATAR(id)} />
        <span className="font-medium text-foreground text-sm">{label}</span>
    </HoverCardTrigger>
);

/** Default padded content — a short operator stat block. */
export const StatBlock = () => (
    <div className="flex h-96 w-full items-start justify-center pt-2">
        <HoverCard open>
            {chip("char_4045_heidi", "Heidi")}
            <HoverCardContent>
                <div className="flex flex-col gap-2">
                    <span className="font-semibold text-sm">Heidi</span>
                    <span className="text-muted-foreground text-xs">5★ Supporter · Bard · Rhodes Island</span>
                    <dl className="grid grid-cols-2 gap-1 font-mono text-xs tabular-nums">
                        <dt className="text-muted-foreground">ATK</dt>
                        <dd className="text-right text-foreground">462</dd>
                        <dt className="text-muted-foreground">Cost</dt>
                        <dd className="text-right text-foreground">13</dd>
                    </dl>
                </div>
            </HoverCardContent>
        </HoverCard>
    </div>
);

/** `p-0` hands the whole surface to a full-bleed preview panel. */
export const FullBleed = () => (
    <div className="flex h-96 w-full items-start justify-center pt-2">
        <HoverCard open>
            {chip("char_002_amiya", "Amiya")}
            <HoverCardContent className="w-max max-w-85 p-0">
                <div className="flex w-64 flex-col overflow-hidden rounded-lg">
                    <div className="flex items-center gap-3 border-b bg-muted p-3">
                        <img alt="" className="size-10 rounded-md object-cover" src={AVATAR("char_002_amiya")} />
                        <div className="min-w-0">
                            <div className="truncate font-semibold text-sm leading-tight">Amiya</div>
                            <div className="text-xs" style={{ color: "#f7e79e" }}>
                                ★★★★★
                            </div>
                        </div>
                    </div>
                    <div className="flex flex-col gap-1.5 p-3 text-xs">
                        <div className="flex justify-between gap-2">
                            <span className="text-muted-foreground">Class</span>
                            <span className="text-foreground">Caster · Splash</span>
                        </div>
                        <div className="flex justify-between gap-2">
                            <span className="text-muted-foreground">Faction</span>
                            <span className="text-foreground">Rhodes Island</span>
                        </div>
                        <div className="flex justify-between gap-2">
                            <span className="text-muted-foreground">Race</span>
                            <span className="text-foreground">Cautus</span>
                        </div>
                    </div>
                </div>
            </HoverCardContent>
        </HoverCard>
    </div>
);

/** Narrow content aligned to the trigger's start edge. */
export const AlignedStart = () => (
    <div className="flex h-96 w-full items-start justify-center pt-2">
        <HoverCard open>
            {chip("char_102_texas", "Texas")}
            <HoverCardContent align="start" className="w-56" sideOffset={8}>
                <div className="flex flex-col gap-2">
                    <span className="font-semibold text-sm">Texas</span>
                    <p className="text-foreground text-xs leading-snug">Generates 1 DP on deployment. A staple opener for low-cost 3-star clears.</p>
                </div>
            </HoverCardContent>
        </HoverCard>
    </div>
);
