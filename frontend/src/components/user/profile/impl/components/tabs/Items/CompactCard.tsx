import { Dialog, DialogTrigger } from "#/components/ui/dialog";
import { RARITY_COLORS } from "./helpers";
import { ItemDialog } from "./ItemDialog";
import { ItemIcon } from "./ItemIcon";
import type { IItemEntry } from "./types";

interface ICompactCardProps {
    item: IItemEntry;
}

function formatCompactQty(n: number): string {
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}m`;
    if (n >= 10_000) return `${Math.round(n / 1000)}k`;
    if (n >= 1_000) return `${(n / 1000).toFixed(1)}k`;
    return String(n);
}

export function CompactCard({ item }: ICompactCardProps) {
    const color = RARITY_COLORS[item.rarityNum] ?? "#b5b5b5";
    const nameIsLong = item.name.length > 14;

    return (
        <Dialog>
            <DialogTrigger
                render={
                    <button
                        type="button"
                        aria-label={`Open details for ${item.name}`}
                        title={item.name}
                        className="fade-in slide-in-from-bottom-2 group relative flex animate-in cursor-pointer flex-col rounded bg-card text-left transition-transform hover:scale-[1.04] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                        style={{ padding: "4px 6px", boxShadow: "0 1px 3px 0 rgb(0 0 0 / 0.1)" }}
                    />
                }
            >
                <div className="mb-0.5 ml-px flex h-4 flex-col justify-center text-left">
                    <span className="truncate text-foreground leading-none" style={{ fontSize: nameIsLong ? "9px" : "11px" }}>
                        {item.name}
                    </span>
                </div>
                <div className="relative box-content aspect-square w-full overflow-hidden" style={{ borderBottom: `3px solid ${color}` }}>
                    <ItemIcon item={item} className="h-full! w-full! rounded-none!" />
                </div>
                <span className="-bottom-1.5 -right-1.5 absolute z-10 rounded-md bg-background/95 px-1.5 py-0.5 font-mono font-bold text-[11px] text-foreground leading-none tabular-nums backdrop-blur-sm" style={{ boxShadow: `inset 0 0 0 1px ${color}80, 0 2px 6px rgba(0,0,0,0.5)` }}>
                    {formatCompactQty(item.quantity)}
                </span>
            </DialogTrigger>
            <ItemDialog item={item} />
        </Dialog>
    );
}
