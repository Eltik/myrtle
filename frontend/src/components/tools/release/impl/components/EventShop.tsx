import { ChevronDown } from "lucide-react";
import * as React from "react";
import { useFormatters, useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { EventShop as EventShopData } from "#/types/generated/EventShop";
import type { ShopGood } from "#/types/generated/ShopGood";
import { formatDateRange } from "../helpers";
import type { messages as helperMessages } from "../helpers.messages";
import { SHOP_KIND_LABEL_KEYS, shopBuyout, shopGroups } from "../plan";
import type { messages as planMessages } from "../plan.messages";
import type { messages } from "./EventShop.messages";
import { useArt } from "./shared";

type EventShopT = TypedT<typeof messages & typeof helperMessages & typeof planMessages>;

export function EventShop({ event }: { event: { shop: EventShopData | null; missionTokens: number } }): React.ReactElement {
    const t: EventShopT = useT("tools");
    const f = useFormatters();
    const locale = useLocale();
    const [open, setOpen] = React.useState(false);
    const { shop, missionTokens } = event;
    const buyout = shopBuyout(event);

    if (!shop || !buyout) return <p className="m-0 font-sans text-[11.5px] text-muted-foreground">{t("release.events.shop.noShop", { count: missionTokens })}</p>;

    return (
        <div className="flex flex-col gap-2 border-border/40 border-t pt-3">
            <button type="button" aria-expanded={open} onClick={() => setOpen((value) => !value)} className="flex w-full cursor-pointer items-center gap-1.5 text-left">
                <ShopIcon item={shop.token} size="size-4" />
                <span className="font-sans font-semibold text-[12px] text-foreground">{t("release.events.shop")}</span>
                <span className="font-mono text-[11px] text-muted-foreground tabular-nums">· {t("release.events.shop.toFarmValue", { sanity: f.number(buyout.sanity) })}</span>
                <ChevronDown className={cn("ml-auto size-4 text-muted-foreground transition-transform", open && "rotate-180")} />
            </button>
            {open && (
                <div className="flex flex-col gap-2">
                    <span className="font-sans text-[11.5px] text-muted-foreground">{t("release.events.shop.meta", { name: shop.shopName, server: shop.server.toUpperCase(), dates: formatDateRange(shop.startTime, shop.endTime, locale, t) })}</span>
                    <dl className="m-0 grid grid-cols-[minmax(0,1fr)_auto] gap-x-4 gap-y-0.5 font-mono text-[11.5px] tabular-nums">
                        <dt className="flex items-center gap-1.5 font-sans text-muted-foreground">
                            <ShopIcon item={shop.token} size="size-4" />
                            {t("release.events.shop.buyEverything")} <span className="text-[10px]">{t("release.events.shop.limitedGoods", { count: buyout.limitedGoods })}</span>
                        </dt>
                        <dd className="m-0 text-right text-foreground">{f.number(buyout.total)}</dd>
                        <dt className="font-sans text-muted-foreground">{t("release.events.shop.missions")}</dt>
                        <dd className="m-0 text-right text-emerald-500">{t("release.events.shop.missionsValue", { count: f.number(buyout.missions) })}</dd>
                        <dt className="font-sans font-semibold text-foreground">{t("release.events.shop.toFarm")}</dt>
                        <dd className="m-0 text-right font-semibold text-foreground">{t("release.events.shop.toFarmValue", { sanity: f.number(buyout.sanity) })}</dd>
                    </dl>
                    <p className="m-0 font-sans text-[10.5px] text-muted-foreground">{t("release.events.shop.sanityNote")}</p>
                    <ShopGoods shop={shop} t={t} />
                </div>
            )}
        </div>
    );
}

function ShopGoods({ shop, t }: { shop: EventShopData; t: EventShopT }): React.ReactElement {
    const f = useFormatters();
    const { limited, unlimited } = shopGroups(shop);
    const token = shop.token.nameEn ?? shop.token.name;
    return (
        <div className="flex flex-col gap-3 pt-1">
            {limited.map((group) => (
                <div key={group.kind} className="flex flex-col gap-1">
                    <div className="flex items-baseline justify-between gap-2">
                        <span className="font-sans font-semibold text-[11.5px] text-foreground">
                            {t(SHOP_KIND_LABEL_KEYS[group.kind])} <span className="font-medium font-mono text-[10px] text-muted-foreground">{t("release.events.shop.groupCount", { count: group.goods.length })}</span>
                        </span>
                        <span className="font-mono text-[11px] text-muted-foreground tabular-nums">{f.number(group.tokens)}</span>
                    </div>
                    <ul className="m-0 grid list-none grid-cols-1 gap-x-4 gap-y-0.5 p-0 sm:grid-cols-2">
                        {group.goods.map((good) => (
                            <ShopGoodRow key={good.goodId} good={good} t={t} />
                        ))}
                    </ul>
                </div>
            ))}
            {unlimited.length > 0 && (
                <div className="flex flex-col gap-1">
                    <span className="font-sans font-semibold text-[11.5px] text-foreground">
                        {t("release.events.shop.unlimited")} <span className="font-medium font-mono text-[10px] text-muted-foreground">{t("release.events.shop.unlimitedNote", { token })}</span>
                    </span>
                    <ul className="m-0 grid list-none grid-cols-1 gap-x-4 gap-y-0.5 p-0 sm:grid-cols-2">
                        {unlimited.map((good) => (
                            <ShopGoodRow key={good.goodId} good={good} t={t} />
                        ))}
                    </ul>
                </div>
            )}
        </div>
    );
}

function ShopGoodRow({ good, t }: { good: ShopGood; t: EventShopT }): React.ReactElement {
    const f = useFormatters();
    const name = good.item.nameEn ?? good.item.name;
    const cnOnly = good.item.nameEn === null;
    return (
        <li className="flex min-w-0 items-center gap-2 font-mono text-[11px] tabular-nums">
            <ShopIcon item={good.item} size="size-6" />
            <span className="min-w-0 flex-1 truncate font-sans text-[11.5px] text-foreground" lang={cnOnly ? "zh-CN" : undefined} translate={cnOnly ? "yes" : undefined} title={name}>
                {name}
                {good.count > 1 && <span className="text-muted-foreground">{t("release.events.shop.goodCount", { count: f.number(good.count) })}</span>}
            </span>
            <span className="shrink-0 text-muted-foreground">
                {f.number(good.price)}
                {good.availCount > 0 ? t("release.events.shop.stock", { count: good.availCount }) : ""}
            </span>
            {good.availCount > 0 && <span className="w-12 shrink-0 text-right text-foreground">{f.number(good.price * good.availCount)}</span>}
        </li>
    );
}

function ShopIcon({ item, size }: { item: ShopGood["item"]; size: string }): React.ReactElement {
    const art = useArt(item.iconPath);
    return art.src ? <img src={art.src} alt="" loading="lazy" onError={art.onError} className={cn("shrink-0 rounded-sm bg-black/30 object-contain", size)} /> : <span aria-hidden="true" className={cn("shrink-0 rounded-sm bg-muted", size)} />;
}
