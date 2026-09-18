import { Badge } from "#/components/ui/badge";
import { DialogDescription, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { CATEGORY_ITEM_LABELS, formatItemType, formatVoucherId, RARITY_COLORS } from "./helpers";
import type { messages as helperMessages } from "./helpers.messages";
import type { messages } from "./ItemDialog.messages";
import { ItemIcon } from "./ItemIcon";
import type { IItemEntry } from "./types";

/** The category names are declared in `helpers.messages.ts`. */
type DialogT = TypedT<typeof messages & typeof helperMessages>;

interface IItemDialogProps {
    item: IItemEntry;
}

export function ItemDialog({ item }: IItemDialogProps) {
    const t: DialogT = useT("user");
    const f = useFormatters();
    const color = RARITY_COLORS[item.rarityNum] ?? "#b5b5b5";
    const meta = item.meta;
    const obtain =
        meta?.obtainApproach
            ?.split("/")
            .map((s) => s.trim())
            .filter(Boolean) ?? [];
    const recipes = meta?.buildingProductList ?? [];
    const vouchers = meta?.voucherRelateList ?? [];
    const itemTypeLabel = formatItemType(meta?.itemType);

    return (
        <DialogPopup className="max-w-2xl">
            <div aria-hidden className="pointer-events-none absolute inset-x-0 top-0 h-32 rounded-t-2xl opacity-40" style={{ background: `radial-gradient(ellipse 80% 100% at 50% 0%, ${color}30, transparent 70%)` }} />
            <DialogHeader className="relative flex-row items-start gap-4 pr-12">
                <div className="shrink-0 overflow-hidden rounded-lg" style={{ borderBottom: `4px solid ${color}` }}>
                    <ItemIcon item={item} size={96} />
                </div>
                <div className="min-w-0 flex-1">
                    <div className="kicker flex flex-wrap items-center gap-x-2 gap-y-1">
                        <span>{t(CATEGORY_ITEM_LABELS[item.category])}</span>
                        <span aria-hidden className="opacity-40">
                            ·
                        </span>
                        <span style={{ color }}>{item.rarityNum}★</span>
                        <span aria-hidden className="opacity-40">
                            ·
                        </span>
                        <span className="font-mono text-muted-foreground/80 normal-case tracking-normal">{item.item_id}</span>
                    </div>
                    <DialogTitle className="mt-1.5 text-2xl">{item.name}</DialogTitle>
                    {meta?.description && <DialogDescription className="mt-2 text-[13.5px] leading-relaxed">{meta.description}</DialogDescription>}
                    {meta?.usage && (
                        <p className="mt-1 text-[13.5px] text-muted-foreground leading-relaxed">
                            <span className="text-foreground">{t("profile.items.dialog.usage")}</span>
                            {meta.usage}
                        </p>
                    )}
                </div>
            </DialogHeader>

            <DialogPanel className="flex flex-col gap-5">
                <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
                    <DialogStat kicker={t("profile.items.dialog.onHand")} value={f.number(item.quantity)} />
                    <DialogStat kicker={t("profile.items.dialog.rarity")} value={`${item.rarityNum}★`} valueColor={color} />
                    {item.expValue != null ? <DialogStat kicker={t("profile.items.dialog.expValue")} value={f.number(item.expValue)} valueColor="var(--success)" /> : <DialogStat kicker={t("profile.items.dialog.type")} value={itemTypeLabel} subtle />}
                    <DialogStat kicker={t("profile.items.dialog.category")} value={t(CATEGORY_ITEM_LABELS[item.category])} subtle />
                </div>

                {recipes.length > 0 && (
                    <section className="flex flex-col gap-2.5">
                        <span className="kicker">{t("profile.items.dialog.craftedIn")}</span>
                        <div className="flex flex-wrap gap-1.5">
                            {recipes.map((r) => (
                                <Badge key={`${r.roomType}-${r.formulaId}`} variant="outline" size="lg" className="gap-1.5 font-normal">
                                    <span className="font-mono text-[10.5px] text-muted-foreground uppercase tracking-wider">{r.roomType}</span>
                                    <span className="font-mono tabular-nums">#{r.formulaId}</span>
                                </Badge>
                            ))}
                        </div>
                    </section>
                )}

                {vouchers.length > 0 && (
                    <section className="flex flex-col gap-2.5">
                        <span className="kicker">{t("profile.items.dialog.voucherExchange")}</span>
                        <div className="flex flex-wrap gap-1.5">
                            {vouchers.map((v) => (
                                <Badge key={v.voucherId} variant="outline" className="font-normal">
                                    {formatVoucherId(v.voucherId)}
                                </Badge>
                            ))}
                        </div>
                    </section>
                )}

                {obtain.length > 0 && (
                    <section className="flex items-center justify-between gap-3">
                        <span className="kicker">{t("profile.items.dialog.obtainChannels")}</span>
                        <div className="flex flex-wrap justify-end gap-1.5">
                            {obtain.map((s) => (
                                <Badge key={s} variant="secondary" className="font-normal">
                                    {s}
                                </Badge>
                            ))}
                        </div>
                    </section>
                )}
            </DialogPanel>
        </DialogPopup>
    );
}

function DialogStat({ kicker, value, valueColor, subtle }: { kicker: string; value: string; valueColor?: string; subtle?: boolean }) {
    return (
        <div className="flex flex-col gap-1.5 rounded-lg border border-border bg-muted/50 p-3">
            <span className="kicker">{kicker}</span>
            <div className="font-bold tabular-nums leading-none" style={{ fontSize: subtle ? "14px" : "18px", color: valueColor ?? "var(--foreground)" }}>
                {value}
            </div>
        </div>
    );
}
