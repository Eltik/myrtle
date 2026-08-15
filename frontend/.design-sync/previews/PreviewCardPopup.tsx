import { PreviewCard, PreviewCardPopup, PreviewCardTrigger } from "frontend";

const AVATAR = (id: string) => `https://api.myrtle.moe/api/avatar/${id}`;

function Trigger({ id, name, sub }: { id: string; name: string; sub: string }) {
    return (
        <PreviewCardTrigger className="block">
            <div className="flex w-40 items-center gap-2 rounded-md border bg-card px-2 py-2 text-left">
                <img alt="" className="size-8 shrink-0 rounded-md object-cover" src={AVATAR(id)} />
                <span className="min-w-0 flex-1">
                    <span className="block truncate font-medium text-foreground text-sm leading-tight">{name}</span>
                    <span className="block text-muted-foreground text-xs">{sub}</span>
                </span>
            </div>
        </PreviewCardTrigger>
    );
}

/** Default popup — centered under the trigger, padded, 16rem wide. */
export const Summary = () => (
    <div className="flex h-96 w-full items-start justify-center pt-2">
        <PreviewCard open>
            <Trigger id="char_4064_mlynar" name="Młynar" sub="6★ · Guard" />
            <PreviewCardPopup>
                <div className="flex flex-col gap-2">
                    <span className="font-semibold text-sm">Młynar</span>
                    <div className="flex items-center gap-1.5 text-muted-foreground text-xs">
                        <span className="text-foreground">Guard</span>
                        <span aria-hidden="true">·</span>
                        <span>Soloblade</span>
                    </div>
                    <dl className="grid grid-cols-2 gap-1 font-mono text-xs tabular-nums">
                        <dt className="text-muted-foreground">ATK</dt>
                        <dd className="text-right text-foreground">1,088</dd>
                        <dt className="text-muted-foreground">DEF</dt>
                        <dd className="text-right text-foreground">565</dd>
                        <dt className="text-muted-foreground">Cost</dt>
                        <dd className="text-right text-foreground">28</dd>
                    </dl>
                </div>
            </PreviewCardPopup>
        </PreviewCard>
    </div>
);

/** `p-0` + `w-max` lets a full-bleed card own the popup surface. */
export const FlushContent = () => (
    <div className="flex h-96 w-full items-start justify-center pt-2">
        <PreviewCard open>
            <Trigger id="char_263_skadi" name="Skadi" sub="6★ · Guard" />
            <PreviewCardPopup className="w-max max-w-85 p-0">
                <div className="flex w-64 flex-col overflow-hidden rounded-lg">
                    <div className="flex items-center gap-3 border-b bg-muted p-3">
                        <img alt="" className="size-10 rounded-md object-cover" src={AVATAR("char_263_skadi")} />
                        <div className="min-w-0">
                            <div className="truncate font-semibold text-sm leading-tight">Skadi</div>
                            <div className="text-xs" style={{ color: "#f7a452" }}>
                                ★★★★★★
                            </div>
                        </div>
                    </div>
                    <div className="flex flex-col gap-1.5 p-3 text-xs">
                        <div className="flex justify-between gap-2">
                            <span className="text-muted-foreground">Position</span>
                            <span className="text-foreground">Melee</span>
                        </div>
                        <div className="flex justify-between gap-2">
                            <span className="text-muted-foreground">Race</span>
                            <span className="text-foreground">Aegir</span>
                        </div>
                        <div className="flex justify-between gap-2">
                            <span className="text-muted-foreground">Faction</span>
                            <span className="text-foreground">Rhodes Island</span>
                        </div>
                    </div>
                </div>
            </PreviewCardPopup>
        </PreviewCard>
    </div>
);

/** `align="start"` with a wider `sideOffset` — used when the trigger sits at a grid edge. */
export const AlignedStart = () => (
    <div className="flex h-96 w-full items-start justify-center pt-2">
        <PreviewCard open>
            <Trigger id="char_102_texas" name="Texas" sub="5★ · Vanguard" />
            <PreviewCardPopup align="start" sideOffset={10}>
                <div className="flex flex-col gap-2">
                    <span className="font-semibold text-sm">Texas</span>
                    <span className="text-muted-foreground text-xs">5★ Vanguard · Pioneer · Columbia</span>
                    <p className="text-foreground text-xs leading-snug">Generates 1 DP on deployment and recovers DP over time. A staple opener for low-cost 3-star clears.</p>
                </div>
            </PreviewCardPopup>
        </PreviewCard>
    </div>
);
