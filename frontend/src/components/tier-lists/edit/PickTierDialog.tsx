import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { ITierOperator } from "#/lib/api/tier-lists";
import { RARITY_HEX_MUTED } from "#/lib/utils";
import { readableTextColor } from "../detail/contrast";
import type { IEditTier } from "./state";

interface IPickTierDialogProps {
    operator: ITierOperator | null;
    currentTierId: string | null;
    tiers: IEditTier[];
    onClose: () => void;
    onPick: (tierId: string | null) => void;
}

export function PickTierDialog({ operator, currentTierId, tiers, onClose, onPick }: IPickTierDialogProps) {
    const accent = operator ? (RARITY_HEX_MUTED[operator.rarity] ?? RARITY_HEX_MUTED[1]) : null;

    return (
        <Dialog open={operator !== null} onOpenChange={(o) => !o && onClose()}>
            <DialogPopup className="sm:max-w-sm">
                <DialogHeader>
                    <DialogTitle>
                        <span className="inline-flex items-center gap-2.5">
                            {operator && (
                                <span className="inline-flex h-8 w-8 shrink-0 items-center justify-center overflow-hidden rounded-md border-2" style={{ borderColor: accent ?? undefined, background: "var(--muted)" }}>
                                    <OperatorAvatar charId={operator.id} name={operator.name} />
                                </span>
                            )}
                            <span className="font-sans">Place {operator?.name ?? ""}</span>
                        </span>
                    </DialogTitle>
                    <DialogDescription>Pick a tier - or remove from the list. You can also drag operators directly onto a tier.</DialogDescription>
                </DialogHeader>

                <div className="flex flex-col gap-1.5 px-6 pb-2">
                    {tiers.map((t) => {
                        const isCurrent = t.id === currentTierId;
                        const fg = readableTextColor(t.color);
                        return (
                            <button key={t.id} type="button" className="flex items-center gap-3 rounded-lg border border-border px-2.5 py-2 text-left font-sans text-sm transition-colors hover:bg-accent/50 data-[selected=true]:border-ring" data-selected={isCurrent || undefined} onClick={() => onPick(t.id)}>
                                <span
                                    className="flex h-9 w-9 shrink-0 items-center justify-center rounded-md font-sans text-[15px] font-extrabold leading-none tracking-tight"
                                    style={{
                                        background: t.color,
                                        color: fg,
                                        textShadow: fg === "white" ? "0 1px 0 oklch(0 0 0 / 0.25)" : "0 1px 0 oklch(1 0 0 / 0.5)",
                                    }}
                                    aria-hidden="true"
                                >
                                    {t.name.length <= 2 ? t.name : t.name.charAt(0)}
                                </span>
                                <span className="min-w-0 flex-1 truncate">{t.name}</span>
                                <span className="font-mono text-[10.5px] tabular-nums text-muted-foreground">{t.operatorIds.length}</span>
                                {isCurrent && <span className="font-mono text-[10px] font-bold uppercase tracking-wider text-primary">Current</span>}
                            </button>
                        );
                    })}
                </div>

                <DialogFooter className="justify-between sm:justify-between">
                    {currentTierId ? (
                        <Button type="button" variant="destructive-outline" size="sm" onClick={() => onPick(null)}>
                            Unplace
                        </Button>
                    ) : (
                        <span />
                    )}
                    <DialogClose render={<Button type="button" variant="outline" />}>Close</DialogClose>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    );
}
