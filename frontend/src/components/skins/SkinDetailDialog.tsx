import { useState } from "react";
import { skinTexture } from "#/components/operators/detail/impl/assets";
import { DialogClose, DialogContent, DialogTitle } from "#/components/ui/dialog";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { ScrollArea } from "#/components/ui/scroll-area";
import { DynamicArtOverlay } from "#/components/user/profile/impl/DynamicArtOverlay";
import type { ISkinIndexEntry } from "#/lib/api/skins";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./SkinDetailDialog.messages";

export interface ISkinPrice {
    kind: "paid" | "free" | "bundle" | "store";
    label: string | null;
    tooltip: string | null;
}

export interface ISkinDetailContentProps {
    skin: ISkinIndexEntry;
    opName: string;
    skinName: string;
    avatarURL: string;
    server?: "en" | "cn";
    price?: ISkinPrice | null;
    corner?: React.ReactNode;
    extraRows?: React.ReactNode;
    closeLabel?: string;
}

export function SkinDetailContent({ skin, opName, skinName, avatarURL, server, price, corner, extraRows, closeLabel }: ISkinDetailContentProps) {
    const t: TypedT<typeof messages> = useT("skins");
    const f = useFormatters();
    const ds = skin.displaySkin;
    const groupName = ds?.skinGroupName;
    const description = ds?.description ?? ds?.content ?? null;
    const dialog = ds?.dialog ?? null;
    const usage = ds?.usage ?? null;
    const obtain = ds?.obtainApproach ?? null;
    const designers = ds?.designerList ?? null;
    const drawers = ds?.drawerList ?? null;
    const releaseTs = ds?.getTime ? ds.getTime * 1000 : null;
    const heroURL = skinTexture(skin.charId, skin.skinId, server);
    const [dynActive, setDynActive] = useState(false);

    return (
        <DialogContent bottomStickOnMobile={false} initialFocus={false} className="flex h-[92vh] max-h-[92vh] w-full max-w-240 flex-col overflow-hidden p-0" showCloseButton>
            <DialogTitle className="sr-only">{t("detail.srTitle", { op: opName, skin: skinName })}</DialogTitle>
            <div className="grid min-h-0 flex-1 grid-cols-1 grid-rows-[auto_minmax(0,1fr)] md:grid-cols-[5fr_4fr] md:grid-rows-1">
                <div className="relative flex h-[min(56vh,100vw)] items-center justify-center overflow-hidden bg-linear-to-b from-muted/20 to-muted/60 md:h-auto md:border-border/60 md:border-r">
                    <img alt={t("detail.heroAlt", { op: opName, skin: skinName })} className={cn("h-full w-full object-contain object-bottom transition-opacity duration-500", dynActive && "opacity-0")} decoding="async" loading="lazy" onError={(e) => ((e.target as HTMLImageElement).src = avatarURL)} src={heroURL} />
                    <DynamicArtOverlay operatorCode={skin.charId} skinId={skin.skinId} framing="authored" surface="panel" backdrop={heroURL} onActiveChange={setDynActive} />
                    {corner && <span className="absolute top-3 left-3">{corner}</span>}
                </div>
                <ScrollArea className="min-h-0">
                    <div className="flex flex-col gap-4 p-5 sm:p-6">
                        <div className="flex items-start gap-3">
                            <span className="flex h-12 w-12 shrink-0 items-center justify-center overflow-hidden rounded-lg bg-muted/40 font-semibold">
                                <OperatorAvatar charId={skin.charId} name={opName} server={server} />
                            </span>
                            <div className="min-w-0 flex-1">
                                <p className="truncate font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">{opName}</p>
                                <h3 className="wrap-break-word font-heading font-semibold text-lg leading-tight">{skinName}</h3>
                                {groupName && groupName !== skinName && <p className="truncate text-muted-foreground text-xs">{groupName}</p>}
                            </div>
                        </div>

                        <dl className="flex flex-col gap-3 text-sm">
                            {price?.label && (
                                <DetailRow label={t("detail.row.price")}>
                                    <span className={cn("font-semibold", price.kind === "free" && "text-emerald-600 dark:text-emerald-400")}>{price.label}</span>
                                    {price.tooltip && <span className="ml-2 text-muted-foreground text-xs">- {price.tooltip}</span>}
                                </DetailRow>
                            )}
                            {obtain && <DetailRow label={t("detail.row.obtain")}>{obtain}</DetailRow>}
                            {usage && <DetailRow label={t("detail.row.usage")}>{usage}</DetailRow>}
                            {description && <DetailRow label={t("detail.row.description")}>{description}</DetailRow>}
                            {dialog && (
                                <DetailRow label={t("detail.row.dialog")}>
                                    <q className="italic">{dialog}</q>
                                </DetailRow>
                            )}
                            {Boolean(drawers?.length || designers?.length) && (
                                <DetailRow label={t("detail.row.credits")}>
                                    {drawers?.length ? <span>{t("detail.credits.art", { names: drawers.join(", ") })}</span> : null}
                                    {drawers?.length && designers?.length ? " · " : null}
                                    {designers?.length ? <span>{t("detail.credits.design", { names: designers.join(", ") })}</span> : null}
                                </DetailRow>
                            )}
                            {releaseTs && <DetailRow label={t("detail.row.released")}>{f.date(new Date(releaseTs), { year: "numeric", month: "long", day: "numeric" })}</DetailRow>}
                            {extraRows}
                        </dl>

                        <DialogClose className="mt-auto cursor-pointer rounded-md border border-border bg-muted/40 px-3 py-2 font-medium text-xs transition-colors hover:bg-muted md:hidden">{closeLabel ?? t("detail.back")}</DialogClose>
                    </div>
                </ScrollArea>
            </div>
        </DialogContent>
    );
}

export function DetailRow({ label, children }: { label: string; children: React.ReactNode }) {
    return (
        <div className="space-y-1">
            <dt className="font-mono font-semibold text-[10px] text-muted-foreground uppercase tracking-[0.12em]">{label}</dt>
            <dd className="text-foreground/85 text-sm leading-relaxed">{children}</dd>
        </div>
    );
}
