import { Link } from "@tanstack/react-router";
import { useSelector } from "@tanstack/react-store";
import { EyeOffIcon, ImageIcon } from "lucide-react";
import { useState } from "react";
import type { messages as cardListMessages } from "#/components/operators/list/impl/components/OperatorCardList.messages";
import { Badge } from "#/components/ui/badge";
import { Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator } from "#/components/ui/breadcrumb";
import { Button } from "#/components/ui/button";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useDataSaver } from "#/hooks/use-data-saver";
import { useOperatorName } from "#/hooks/use-operator-name";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { themeStore } from "#/lib/theme/store";
import { cn, getAvatarById, nationLabel, professionLabel, rarityToNumber, subProfessionLabel } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";
import { campLogo, operatorHero } from "../assets";
import { RARITY_COLORS, RARITY_GLOW } from "../constants";
import { useParallaxProgress } from "../useParallaxProgress";
import type { messages } from "./OperatorHero.messages";
import styles from "./OperatorHero.module.css";

interface IOperatorHeroProps {
    operator: IOperatorListItem;
}

/**
 * The desktop scrim, page colour at the bottom to clear at the top. The three-stop
 * `from-background via-background/80 to-transparent` it replaces read as horizontal
 * bands across the full width over the art at 1440; nine eased stops at the same
 * coverage (solid at 0, 80% at the middle, clear at the top) step finely enough not to.
 */
const EASED_SCRIM = `linear-gradient(to top, ${[
    [100, 0],
    [96, 15],
    [90, 30],
    [80, 50],
    [62, 62],
    [42, 74],
    [22, 85],
    [8, 93],
    [0, 100],
]
    .map(([alpha, at]) => `color-mix(in oklab, var(--background) ${alpha}%, transparent) ${at}%`)
    .join(", ")})`;

export function OperatorHero({ operator }: IOperatorHeroProps) {
    const t: TypedT<typeof messages & typeof cardListMessages> = useT("operators");
    const ref = useParallaxProgress<HTMLDivElement>();
    const name = useOperatorName()(operator);

    const rarityNum = rarityToNumber(operator.rarity);
    const rarityColor = RARITY_COLORS[operator.rarity] ?? RARITY_COLORS.TIER_1;
    const rarityGlow = RARITY_GLOW[operator.rarity] ?? RARITY_GLOW.TIER_1;

    // The page's LCP. Both breakpoint copies share this URL, so the one request is marked
    // high priority whichever copy the viewport shows.
    const img = operatorHero(operator.id ?? "", operator.skin, operator.portrait, operator.server);
    // Under data saver the splash art waits for a press, remembered per operator so the
    // next operator opened from this page starts without it again.
    const dataSaver = useDataSaver();
    const [artRequestedFor, setArtRequestedFor] = useState<string | null>(null);
    const showArt = !dataSaver || artRequestedFor === operator.id;
    const requestArt = () => setArtRequestedFor(operator.id ?? null);

    const factionId = operator.nationId?.length ? operator.nationId : operator.teamId?.length ? operator.teamId : operator.groupId?.length ? operator.groupId : null;

    return (
        <div className={cn("relative w-full overflow-hidden contain-layout", styles["parallax-hero"])} ref={ref}>
            <div className="md:hidden">
                <div className="relative h-90 w-full overflow-hidden sm:h-100">
                    <div aria-hidden className={cn("backface-hidden absolute inset-x-0 top-0 will-change-transform contain-paint", styles["parallax-image"])}>
                        <div className="flex items-start justify-center pt-0">
                            <div className="relative h-120 w-[85vw] max-w-95 sm:h-135 sm:w-110 sm:max-w-none">{showArt && <img alt={name} className={cn("h-full w-full object-contain object-top", rarityGlow)} decoding="async" fetchPriority="high" height={540} loading="eager" src={img} width={440} />}</div>
                        </div>
                    </div>
                    {!showArt && <ArtPlaceholder operator={operator} onLoad={requestArt} className="inset-x-0 top-6 sm:top-8" />}
                    {/* The text block is about 136px tall, 166px when the badges wrap (summed from its
                        utilities, not measured in a browser). The old scrim was h-1/2 with its 80% stop at
                        90px, so the breadcrumb sat on 39 to 53% of the page colour (12 to 27% wrapped) over
                        the art: muted text measured 1.21 to 2.01 there over a black pixel in light mode.
                        Solid to 173px of 240 covers the wrapped block at 7.15; the upper 120px of the
                        360px frame stays bare art and the next 67px fade in over it. */}
                    <div className="pointer-events-none absolute inset-x-0 bottom-0 h-60 bg-linear-to-t from-72% from-background via-85% via-background/85 to-transparent" />
                    <div className={cn("backface-hidden relative z-10 flex h-full flex-col justify-end px-3 pb-4 will-change-[transform,opacity] sm:px-4 sm:pb-5", styles["parallax-content"])}>
                        <Breadcrumb className="mb-2">
                            <BreadcrumbList className="text-xs">
                                <BreadcrumbItem>
                                    <BreadcrumbLink render={<Link to="/operators" />}>{t("hero.breadcrumb.operators")}</BreadcrumbLink>
                                </BreadcrumbItem>
                                <BreadcrumbSeparator />
                                <BreadcrumbItem>
                                    <BreadcrumbPage className="font-medium">{name}</BreadcrumbPage>
                                </BreadcrumbItem>
                            </BreadcrumbList>
                        </Breadcrumb>

                        <div>
                            <h1 className="font-bold text-2xl text-foreground tracking-tight sm:text-3xl">{name}</h1>
                            <div className="mt-1.5 flex items-center gap-2">
                                <span aria-label={t("card.rarityStars", { rarity: rarityNum })} className={cn("font-semibold text-base tracking-wider", rarityColor)} role="img">
                                    {"★".repeat(rarityNum)}
                                </span>
                                <span aria-hidden="true" className="text-subtle-foreground">
                                    |
                                </span>
                                <span className="text-muted-foreground text-sm">{subProfessionLabel(operator)}</span>
                            </div>
                        </div>

                        <div className="mt-3 flex flex-wrap gap-2">
                            <Badge variant="outline" className="border-transparent bg-accent text-foreground">
                                {professionLabel(operator)}
                            </Badge>
                            <Badge variant="outline" className="border-transparent bg-accent text-foreground">
                                {operator.position === "RANGED" ? t("hero.position.ranged") : operator.position === "MELEE" ? t("hero.position.melee") : operator.position}
                            </Badge>
                            {operator.nationId && (
                                <Badge variant="outline" className="border-transparent bg-accent text-foreground">
                                    {nationLabel(operator)}
                                </Badge>
                            )}
                            <HiddenFactionBadges operator={operator} />
                        </div>
                    </div>
                </div>
            </div>

            <div className="hidden md:block">
                <div className="relative h-95 w-full overflow-hidden lg:h-105">
                    <div aria-hidden className={cn("backface-hidden absolute inset-x-0 top-0 will-change-transform contain-paint", styles["parallax-image"])}>
                        <div className="flex items-start justify-end pr-[5%] lg:pr-[10%]">
                            <div className="relative h-155 w-130 lg:h-180 lg:w-150">{showArt && <img alt={name} className={cn("h-full w-full object-contain object-top", rarityGlow)} decoding="async" fetchPriority="high" height={720} loading="eager" src={img} width={600} />}</div>
                        </div>
                    </div>
                    {!showArt && <ArtPlaceholder operator={operator} onLoad={requestArt} className="top-12 right-[5%] w-130 lg:right-[10%] lg:w-150" />}
                    <div className="pointer-events-none absolute inset-x-0 bottom-0 h-1/3" style={{ backgroundImage: EASED_SCRIM }} />
                    <div className={cn("backface-hidden relative z-10 mx-auto flex h-full max-w-6xl flex-col justify-end px-8 pb-6 will-change-[transform,opacity] lg:pb-8", styles["parallax-content"])}>
                        <Breadcrumb className="mb-3">
                            <BreadcrumbList>
                                <BreadcrumbItem>
                                    <BreadcrumbLink render={<Link to="/operators" />}>{t("hero.breadcrumb.operators")}</BreadcrumbLink>
                                </BreadcrumbItem>
                                <BreadcrumbSeparator />
                                <BreadcrumbItem>
                                    <BreadcrumbPage className="font-medium">{name}</BreadcrumbPage>
                                </BreadcrumbItem>
                            </BreadcrumbList>
                        </Breadcrumb>

                        <div className="flex flex-row items-end justify-between gap-4">
                            <div className="flex flex-col gap-2">
                                <div>
                                    <h1 className="font-bold text-4xl text-foreground tracking-tight lg:text-5xl">{name}</h1>
                                    <div className="mt-1.5 flex items-center gap-3">
                                        <span aria-label={t("card.rarityStars", { rarity: rarityNum })} className={cn("font-semibold text-lg tracking-wider", rarityColor)} role="img">
                                            {"★".repeat(rarityNum)}
                                        </span>
                                        <span aria-hidden="true" className="text-subtle-foreground">
                                            |
                                        </span>
                                        <span className="text-base text-muted-foreground">{subProfessionLabel(operator)}</span>
                                    </div>
                                </div>

                                <div className="flex flex-wrap gap-2">
                                    <Badge variant="outline" className="border-transparent bg-accent text-foreground">
                                        {professionLabel(operator)}
                                    </Badge>
                                    <Badge variant="outline" className="border-transparent bg-accent text-foreground">
                                        {operator.position === "RANGED" ? t("hero.position.ranged") : operator.position === "MELEE" ? t("hero.position.melee") : operator.position}
                                    </Badge>
                                    {operator.nationId && (
                                        <Badge variant="outline" className="border-transparent bg-accent text-foreground">
                                            {nationLabel(operator)}
                                        </Badge>
                                    )}
                                    <HiddenFactionBadges operator={operator} />
                                </div>
                            </div>

                            {factionId && (
                                <div className="flex items-center gap-4">
                                    <div className="h-14 w-14 lg:h-16 lg:w-16">
                                        <img alt="" className="h-full w-full object-contain opacity-80" decoding="async" height={64} loading="lazy" src={campLogo(factionId, operator.server)} width={64} />
                                    </div>
                                </div>
                            )}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}

/**
 * Stands in for the splash art while data saver is on: the operator's deployment avatar,
 * a few dozen KB against the art's megabytes, and the button that loads the art. Sits
 * above the hero's text layer so the button takes the press.
 */
function ArtPlaceholder({ operator, onLoad, className }: { operator: IOperatorListItem; onLoad: () => void; className?: string }) {
    const t: TypedT<typeof messages> = useT("operators");
    return (
        <div className={cn("absolute z-20 flex flex-col items-center gap-3", className)}>
            {operator.id && <img alt="" className="size-24 rounded-xl border border-border/60 bg-card/60 object-contain sm:size-28" decoding="async" src={getAvatarById(operator.id, operator.server)} />}
            <Button variant="outline" size="sm" onClick={onLoad}>
                <ImageIcon aria-hidden />
                {t("hero.dataSaver.load")}
            </Button>
            <span className="font-mono text-[10px] text-muted-foreground uppercase tracking-wider">{t("hero.dataSaver.note")}</span>
        </div>
    );
}

/**
 * The factions an operator belongs to that the game never puts on their card
 * (`hiddenFactions`, the `SubPower` ids outside `MainPower`), behind the
 * reader's spoiler setting. The setting lives in localStorage, so SSR and the
 * first client render both read it as off and the badges arrive after
 * hydration, never as a mismatch.
 */
function HiddenFactionBadges({ operator }: { operator: IOperatorListItem }) {
    const t: TypedT<typeof messages> = useT("operators");
    const spoilers = useSelector(themeStore, (s) => s.spoilers);
    const factions = operator.hiddenFactions ?? [];
    if (!spoilers || factions.length === 0) return null;
    return factions.map((faction) => (
        <Tooltip key={faction.id}>
            <TooltipTrigger render={<Badge variant="warning" className="gap-1.5 border-warning/50 border-dashed" tabIndex={0} />}>
                <EyeOffIcon aria-hidden className="size-3" />
                <CampLogo id={faction.id} server={operator.server} />
                <span className="sr-only">{t("hero.hiddenFaction.srLabel")}</span>
                {faction.name ?? faction.id}
            </TooltipTrigger>
            <TooltipPopup>{t("hero.hiddenFaction.tooltip")}</TooltipPopup>
        </Tooltip>
    ));
}

/**
 * Not every faction has a camp logo, and a few logo files are capitalised
 * (`logo_Laterano`), so a logo that fails to load is dropped instead of
 * leaving a broken image in the badge.
 */
function CampLogo({ id, server }: { id: string; server: IOperatorListItem["server"] }) {
    const [failed, setFailed] = useState(false);
    if (failed) return null;
    return <img alt="" className="size-3.5 object-contain" decoding="async" loading="lazy" onError={() => setFailed(true)} src={campLogo(id, server)} />;
}
