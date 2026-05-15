import { Badge } from "#/components/ui/badge";
import { Card } from "#/components/ui/card";
import { Dialog, DialogTrigger } from "#/components/ui/dialog";
import { CATEGORY_ITEM_LABELS, RARITY_COLORS } from "./helpers";
import { ItemDialog } from "./ItemDialog";
import { ItemIcon } from "./ItemIcon";
import type { IItemEntry } from "./types";

interface IDetailedCardProps {
    item: IItemEntry;
}

export function DetailedCard({ item }: IDetailedCardProps) {
    const color = RARITY_COLORS[item.rarityNum] ?? "#b5b5b5";
    const obtain =
        item.meta?.obtainApproach
            ?.split("/")
            .map((s) => s.trim())
            .filter(Boolean) ?? [];
    const stageCount = item.meta?.stageDropList?.length ?? 0;
    const recipeCount = item.meta?.buildingProductList?.length ?? 0;

    return (
        <Dialog>
            <DialogTrigger
                render={
                    <Card
                        aria-label={`Open details for ${item.name}`}
                        className="fade-in slide-in-from-bottom-4 group relative animate-in cursor-pointer overflow-hidden rounded-xl! text-left transition-all duration-200 hover:-translate-y-0.5 hover:border-foreground/20 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                        render={<button type="button" />}
                    />
                }
            >
                <div aria-hidden className="absolute inset-x-0 top-0 z-10 h-px" style={{ background: `linear-gradient(to right, transparent, ${color}, transparent)` }} />
                <div aria-hidden className="pointer-events-none absolute inset-x-0 top-0 h-20 opacity-50" style={{ background: `radial-gradient(ellipse 60% 100% at 50% 0%, ${color}26, transparent 70%)` }} />

                <div className="relative flex items-start justify-between gap-2 px-4 pt-3.5 pb-1">
                    <div className="flex min-w-0 items-center gap-2">
                        <Badge size="sm" variant="outline" className="font-mono tabular-nums" style={{ background: `${color}25`, color, borderColor: `${color}66` }}>
                            {item.rarityNum}★
                        </Badge>
                        <span className="truncate font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">{CATEGORY_ITEM_LABELS[item.category]}</span>
                    </div>
                    {(stageCount > 0 || recipeCount > 0) && (
                        <span className="inline-flex shrink-0 items-center gap-2 font-mono text-[10.5px] text-muted-foreground tabular-nums">
                            {stageCount > 0 && (
                                <span title={`${stageCount} drop stage${stageCount === 1 ? "" : "s"}`}>
                                    {stageCount}
                                    <span className="ml-0.5 opacity-60">drops</span>
                                </span>
                            )}
                            {recipeCount > 0 && (
                                <span className="text-foreground/70" title="Has crafting recipe">
                                    ⚒
                                </span>
                            )}
                        </span>
                    )}
                </div>

                <h3 className="relative px-4 pb-3 font-semibold text-[15.5px] text-foreground leading-tight tracking-tight">{item.name}</h3>

                <div className="relative flex items-center gap-3.5 px-4 pb-3">
                    <ItemIcon item={item} size={64} />
                    <div className="flex min-w-0 flex-1 flex-col gap-0.5">
                        <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">Quantity</span>
                        <span className="font-bold tabular-nums leading-none" style={{ fontSize: "1.875rem", letterSpacing: "-0.02em" }}>
                            {item.quantity.toLocaleString()}
                        </span>
                        {item.meta?.usage && <span className="mt-1 line-clamp-2 text-[11.5px] text-muted-foreground leading-tight">{item.meta.usage}</span>}
                    </div>
                </div>

                {obtain.length > 0 && (
                    <div className="relative flex items-center gap-2 border-border border-t px-4 py-2.5">
                        <span className="shrink-0 font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">Obtain</span>
                        <div className="flex min-w-0 flex-1 items-center gap-1.5 overflow-hidden">
                            {obtain.slice(0, 2).map((c) => (
                                <Badge key={c} size="sm" variant="secondary" className="truncate font-normal">
                                    {c}
                                </Badge>
                            ))}
                            {obtain.length > 2 && <span className="shrink-0 font-mono text-[10.5px] text-muted-foreground tabular-nums">+{obtain.length - 2}</span>}
                        </div>
                    </div>
                )}

                <div aria-hidden className="absolute inset-x-0 bottom-0 z-10 h-1" style={{ background: color }} />
            </DialogTrigger>
            <ItemDialog item={item} />
        </Dialog>
    );
}
