import { Badge, PreviewCard, PreviewCardPopup, PreviewCardTrigger } from "frontend";

const AVATAR = (id: string) => `https://api.myrtle.moe/api/avatar/${id}`;

const TAGS = ["Top Operator", "Melee", "DPS", "Survival", "Nuker"];

/** Trigger wrapping a tile — `className="block"` keeps the tile's own layout. */
export const TileTrigger = () => (
    <div className="flex h-96 w-full items-start justify-center pt-2">
        <PreviewCard open>
            <PreviewCardTrigger className="block">
                <div className="flex w-32 flex-col rounded bg-card pt-1 pr-2 pb-1 pl-1.5">
                    <span className="ml-px truncate text-foreground text-xs leading-tight">Młynar</span>
                    <div className="relative box-content aspect-square h-20 overflow-hidden" style={{ borderBottom: "4px solid #f7a452" }}>
                        <img alt="Młynar" className="relative h-full w-full object-contain" src={AVATAR("char_4064_mlynar")} />
                    </div>
                    <div className="mt-0.5 truncate text-center text-muted-foreground text-xs leading-tight">Soloblade</div>
                </div>
            </PreviewCardTrigger>
            <PreviewCardPopup>
                <div className="flex flex-col gap-1">
                    <span className="font-semibold text-sm">Młynar</span>
                    <span className="text-muted-foreground text-xs">6★ Guard · Soloblade · Kazimierz</span>
                </div>
            </PreviewCardPopup>
        </PreviewCard>
    </div>
);

/** `render` swaps the trigger element — here the recruitment result row button. */
export const RenderedAsButton = () => (
    <div className="flex h-96 w-full items-start justify-center pt-2">
        <PreviewCard open>
            <PreviewCardTrigger
                render={
                    <button className="flex w-72 cursor-pointer items-center gap-2 rounded-md border bg-card px-2 py-2 text-left" type="button">
                        <span aria-hidden="true" className="inline-flex size-8 shrink-0 items-center justify-center overflow-hidden rounded-md bg-muted">
                            <img alt="" className="block h-full w-full object-cover" src={AVATAR("char_263_skadi")} />
                        </span>
                        <span className="min-w-0 flex-1">
                            <span className="block truncate font-medium text-foreground text-sm leading-tight">Skadi</span>
                            <span className="block text-muted-foreground text-xs">6★ · Guard</span>
                        </span>
                    </button>
                }
            />
            <PreviewCardPopup className="w-max max-w-72 p-3">
                <div className="flex flex-col gap-2">
                    <div className="font-medium text-muted-foreground text-xs">Tags</div>
                    <div className="flex flex-wrap gap-1">
                        {TAGS.map((tag) => (
                            <Badge key={tag} size="default" variant="outline">
                                {tag}
                            </Badge>
                        ))}
                    </div>
                </div>
            </PreviewCardPopup>
        </PreviewCard>
    </div>
);

/** Trigger inline in prose — a material link inside a stage description. */
export const InlineTrigger = () => (
    <div className="flex h-96 w-full items-start justify-center pt-2">
        <div className="max-w-sm text-foreground text-sm leading-relaxed">
            First-clear of <span className="font-medium">S4-1</span> awards two{" "}
            <PreviewCard open>
                <PreviewCardTrigger className="cursor-pointer font-medium text-primary underline">Chip Catalysts</PreviewCardTrigger>
                <PreviewCardPopup className="w-56">
                    <div className="flex flex-col gap-2">
                        <span className="font-semibold text-sm">Chip Catalyst</span>
                        <span className="text-muted-foreground text-xs">Tier 4 · Elite material</span>
                        <span className="text-foreground text-xs">Converts a class chip pack into any other class. Farmed in Chapter 8 and the weekly supply rotation.</span>
                    </div>
                </PreviewCardPopup>
            </PreviewCard>
            , plus 1,500 LMD.
        </div>
    </div>
);
