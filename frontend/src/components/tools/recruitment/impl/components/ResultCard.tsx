import type * as React from "react";
import { potentialIcon } from "#/components/operators/detail/impl/assets";
import { Badge } from "#/components/ui/badge";
import { Kicker } from "#/components/ui/kicker";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Popover, PopoverPopup, PopoverTrigger } from "#/components/ui/popover";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, RARITY_HEX_MUTED, rarityInk } from "#/lib/utils";
import { guaranteedFloorRarity } from "../calculator";
import { SENIOR_OPERATOR_TAG_ID, TOP_OPERATOR_TAG_ID } from "../constants";
import { getStarsDisplay } from "../helpers";
import { type IRarityNewOdds, newOddsPercent, newOperatorOdds } from "../newOperatorOdds";
import type { IRecruitableOperator, IRosterOverlay, ITagCombinationResult, ResultLayout } from "../types";
import type { messages } from "./ResultCard.messages";

export type ResultT = TypedT<typeof messages>;

/** A 44px-tall tap area on a coarse pointer for the inline hint buttons, without moving the row. */
const INLINE_HIT = "relative rounded-sm focus-visible:outline-2 focus-visible:outline-ring pointer-coarse:after:absolute pointer-coarse:after:inset-x-0 pointer-coarse:after:top-1/2 pointer-coarse:after:h-11 pointer-coarse:after:-translate-y-1/2";

interface IResultCardProps {
    result: ITagCombinationResult;
    roster: IRosterOverlay | null;
}

/**
 * One tag combination as one row: the tags and the guaranteed floor in a narrow
 * left column, the operators as portrait tiles wrapping on the right. Rows sit
 * in one list card (see ResultsList), so a combination costs a line or two of
 * tiles rather than a card of wide rows.
 */
export function ResultCard({ result, roster }: IResultCardProps): React.ReactElement {
    return (
        <div className="grid grid-cols-1 gap-2 px-3 py-2.5 sm:grid-cols-[12rem_1fr] sm:gap-3 sm:px-4">
            <div className="flex min-w-0 flex-wrap content-start items-center gap-1">
                <ResultTagBadges result={result} />
                <span className="basis-full max-sm:hidden" aria-hidden="true" />
                <GuaranteedBadge result={result} layout="compact" />
                {roster?.showNewOdds ? (
                    <>
                        <span className="basis-full" aria-hidden="true" />
                        <NewOddsLabel result={result} roster={roster} />
                    </>
                ) : null}
            </div>
            <ul className="flex min-w-0 flex-wrap content-start gap-1">
                {result.operators.map((op) => (
                    <OperatorTile key={op.id} operator={op} roster={roster} />
                ))}
            </ul>
        </div>
    );
}

/** The two qualification tags that guarantee a rarity carry that rarity's tone, as they do in the tag picker. */
const TAG_RARITY: Partial<Record<number, number>> = { [TOP_OPERATOR_TAG_ID]: 6, [SENIOR_OPERATOR_TAG_ID]: 5 };

/** A combination's tags; foreground on a 12% rarity tint measures 17.73 / 14.94 (6★) and 18.92 / 13.70 (5★), light / dark. */
export function ResultTagBadges({ result }: { result: ITagCombinationResult }): React.ReactElement {
    return (
        <>
            {result.tagNames.map((name, i) => {
                const rarity = TAG_RARITY[result.tags[i] ?? -1];
                return rarity ? (
                    <Badge key={name} variant="tint" size="default" style={{ "--tone": `var(--rarity-${rarity})` } as React.CSSProperties}>
                        {name}
                    </Badge>
                ) : (
                    <Badge key={name} variant="outline" size="default">
                        {name}
                    </Badge>
                );
            })}
        </>
    );
}

/** Rarity 1 is the robot tier, and "★" alone would read as an ordinary 1★. */
function floorLabel(rarity: number, t: ResultT): string {
    return rarity === 1 ? t("recruit.result.robotFloor") : t("recruit.result.starFloor", { rarity });
}

/**
 * The guaranteed worst case, shown on every card because the list is ranked by it (hiding it
 * below a 5★ lock made the ordering depend on a value the reader could not see).
 *
 * A 5★/6★ lock keeps the filled badge: a combination that cannot miss is a different thing.
 * Everything else gets a quiet label (no fill, no border, colour only on the value).
 *
 * `layout` only changes alignment: the compact row pushes the badge right on phones, the
 * detailed card header pushes the quiet label right at every width.
 */
export function GuaranteedBadge({ result, layout }: { result: ITagCombinationResult; layout: ResultLayout }): React.ReactElement {
    const t: ResultT = useT("tools");
    const floor = guaranteedFloorRarity(result);

    if (floor >= 5) {
        return (
            <Badge className={cn("px-1.5", layout === "compact" && "max-sm:ml-auto")} style={{ "--tone": `var(--rarity-${floor})` } as React.CSSProperties} variant="tint">
                {t("recruit.result.guaranteed", { rarity: floor })}
            </Badge>
        );
    }

    // A button with no action of its own, so the kit Tooltip opens on a tap and a keyboard reaches it.
    return (
        <Tooltip>
            <TooltipTrigger render={<button type="button" className={cn("inline-flex h-5.5 shrink-0 cursor-help items-center gap-1 whitespace-nowrap px-0.5 font-medium text-xs sm:h-4.5 sm:text-[11px]", INLINE_HIT, layout === "compact" ? "max-sm:ml-auto" : "ml-auto")} />}>
                <span className="text-muted-foreground">{t("recruit.result.min")}</span>
                <span className="font-mono" style={{ color: rarityInk(floor) }}>
                    {floorLabel(floor, t)}
                </span>
            </TooltipTrigger>
            <TooltipPopup>{t("recruit.result.minTitle", { floor: floorLabel(floor, t) })}</TooltipPopup>
        </Tooltip>
    );
}

/** "50% new", or the two certainties in words, so a reader never has to decode 100% or 0%. */
function oddsText(odds: IRarityNewOdds, withRarity: boolean, t: ResultT): string {
    const pct = newOddsPercent(odds);
    if (!withRarity) {
        if (pct === 100) return t("recruit.result.newOdds.allNew");
        if (pct === 0) return t("recruit.result.newOdds.allOwned");
        return t("recruit.result.newOdds.pct", { pct });
    }
    const rarity = floorLabel(odds.rarity, t);
    if (pct === 100) return t("recruit.result.newOdds.rarityAllNew", { rarity });
    if (pct === 0) return t("recruit.result.newOdds.rarityAllOwned", { rarity });
    return t("recruit.result.newOdds.rarityPct", { rarity, pct });
}

/**
 * The chance the recruit is an operator the roster does not hold. The game rolls rarity first at unpublished rates, so
 * each rarity the pool can produce gets its own exact figure and they are never added up. A pool of one rarity reads as
 * a single figure, the floor badge beside it already names the rarity.
 */
export function NewOddsLabel({ result, roster }: { result: ITagCombinationResult; roster: IRosterOverlay }): React.ReactElement | null {
    const t: ResultT = useT("tools");
    const odds = newOperatorOdds(result.operators, roster.potentialByOperator);
    if (!odds || odds.length === 0) return null;
    const multi = odds.length > 1;
    const lines = odds.map((o) => t("recruit.result.newOdds.title", { rarity: floorLabel(o.rarity, t), unowned: o.unowned, total: o.total }));
    const summary = multi ? [...lines, t("recruit.result.newOdds.unpublished")].join(" ") : lines.join(" ");

    return (
        <Tooltip>
            <TooltipTrigger render={<button type="button" data-slot="recruit-new-odds" aria-label={summary} className={cn("inline-flex min-h-5.5 min-w-0 max-w-full cursor-help flex-wrap items-center gap-x-1 px-0.5 text-left font-medium text-xs sm:min-h-4.5 sm:text-[11px]", INLINE_HIT)} />}>
                {odds.map((o, i) => (
                    <span key={o.rarity} className="inline-flex items-center gap-1 whitespace-nowrap">
                        {i > 0 ? (
                            <span aria-hidden="true" className="text-muted-foreground">
                                ·
                            </span>
                        ) : null}
                        <OddsSegment odds={o} withRarity={multi} t={t} />
                    </span>
                ))}
            </TooltipTrigger>
            <TooltipPopup>
                <div className="flex max-w-64 flex-col gap-1">
                    {lines.map((line) => (
                        <span key={line}>{line}</span>
                    ))}
                    {multi ? <span className="text-muted-foreground">{t("recruit.result.newOdds.unpublished")}</span> : null}
                </div>
            </TooltipPopup>
        </Tooltip>
    );
}

function OddsSegment({ odds, withRarity, t }: { odds: IRarityNewOdds; withRarity: boolean; t: ResultT }): React.ReactElement {
    const pct = newOddsPercent(odds);
    const text = oddsText(odds, withRarity, t);
    return (
        <span className={cn("font-mono", pct === 0 ? "text-muted-foreground" : !withRarity && "text-foreground")} style={pct !== 0 && withRarity ? { color: rarityInk(odds.rarity) } : undefined}>
            {text}
        </span>
    );
}

export function OperatorTagList({ tags }: { tags: string[] }): React.ReactElement {
    const t: ResultT = useT("tools");
    if (tags.length === 0) {
        return <div className="text-[12px] text-muted-foreground italic">{t("recruit.result.noTags")}</div>;
    }
    return (
        <div className="flex flex-wrap items-center gap-1.5">
            {tags.map((tag) => (
                <Badge key={tag} variant="outline" size="default">
                    {tag}
                </Badge>
            ))}
        </div>
    );
}

/** Every attribute a BUFF potential carries in the character table (EN, 2026-09-21); a new one falls through to a raw label. */
const STAT_LABEL_KEY: Record<string, keyof typeof messages> = {
    COST: "recruit.result.upgrade.stat.COST",
    RESPAWN_TIME: "recruit.result.upgrade.stat.RESPAWN_TIME",
    ATK: "recruit.result.upgrade.stat.ATK",
    DEF: "recruit.result.upgrade.stat.DEF",
    MAX_HP: "recruit.result.upgrade.stat.MAX_HP",
    MAGIC_RESISTANCE: "recruit.result.upgrade.stat.MAGIC_RESISTANCE",
    ATTACK_SPEED: "recruit.result.upgrade.stat.ATTACK_SPEED",
};

/** Signed like the game's own potential text: "+28", "-1". Whole numbers only, which is every value in the table. */
function signed(value: number): string {
    return value > 0 ? `+${value}` : `${value}`;
}

type UpgradeTone = "gain" | "unowned" | "maxed";

export const UPGRADE_TONE_CLASS: Record<UpgradeTone, string> = {
    gain: "text-foreground",
    unowned: "text-foreground",
    maxed: "text-muted-foreground",
};

/**
 * The gain from the operator's NEXT potential rank, or what stands in for it:
 * unowned (recruiting is the gain), maxed (nothing left). Stat labels resolve
 * off the attribute type; an attribute the catalog does not know falls back to
 * a raw "ATTRIBUTE +n" so it still reads, just untranslated.
 */
export function nextUpgradeLabel(operator: IRecruitableOperator, potential: number | undefined, t: ResultT): { text: string; tone: UpgradeTone } {
    if (potential === undefined) return { text: t("recruit.result.upgrade.unowned"), tone: "unowned" };
    const next = operator.potentials[potential];
    if (!next) return { text: t("recruit.result.upgrade.maxed"), tone: "maxed" };
    switch (next.kind) {
        case "stat": {
            const key = STAT_LABEL_KEY[next.attribute];
            return { text: key ? t(key, { value: signed(next.value) }) : `${next.attribute} ${signed(next.value)}`, tone: "gain" };
        }
        case "talent":
            return { text: next.of > 1 ? t("recruit.result.upgrade.talent", { n: next.index + 1 }) : t("recruit.result.upgrade.talentOnly"), tone: "gain" };
        case "text":
            return { text: next.text, tone: "gain" };
    }
}

/** 1★ is white in `RARITY_HEX`, which disappears on the light theme; the muted map keeps the robot tier visible. */
function rarityHex(rarity: number): string {
    return RARITY_HEX_MUTED[rarity] ?? RARITY_HEX_MUTED[1] ?? "#b5b5b5";
}

/**
 * One operator as a small portrait tile: rarity-tinted square, name under it,
 * and the next-potential gain under that when the reader asked for it. Every
 * detail the tile leaves out (stars, class, tags) lives in the popover, which
 * opens on hover for a pointer and on tap for touch.
 */
function OperatorTile({ operator, roster }: { operator: IRecruitableOperator; roster: IRosterOverlay | null }): React.ReactElement {
    const t: ResultT = useT("tools");
    const hex = rarityHex(operator.rarity);
    const profession = operator.professionName;

    const potential = roster?.potentialByOperator.get(operator.id);
    const owned = potential !== undefined;
    const showPotential = roster?.showPotentials ?? false;
    const upgrade = roster?.showNextUpgrade ? nextUpgradeLabel(operator, potential, t) : null;
    const potentialLabel = showPotential && owned ? t("recruit.result.potentialAlt", { rank: potential + 1 }) : null;

    return (
        <li className="w-16 sm:w-17">
            <Popover>
                <PopoverTrigger
                    openOnHover
                    delay={150}
                    render={(props) => (
                        <button {...props} type="button" className="flex w-full cursor-pointer flex-col items-center gap-0.5 rounded-md p-0.5 text-center transition-colors hover:bg-accent/40 focus-visible:outline-2 focus-visible:outline-ring data-popup-open:bg-accent/40">
                            <span className="relative block size-11 shrink-0 overflow-hidden rounded-md font-semibold text-[13px] leading-11" style={{ backgroundColor: `${hex}33`, boxShadow: `inset 0 0 0 1px ${hex}73` }}>
                                <span aria-hidden="true" className={cn("block size-full", showPotential && !owned && "opacity-50 grayscale")}>
                                    <OperatorAvatar charId={operator.id} name={operator.name} />
                                </span>
                                <span aria-hidden="true" className="absolute inset-x-0 bottom-0 h-0.75" style={{ backgroundColor: hex }} />
                                {potentialLabel && potential !== undefined && potential > 0 && <img alt={potentialLabel} className="icon-theme-aware absolute bottom-0.5 left-0 h-4 w-3.5 object-contain drop-shadow-sm" decoding="async" height={16} loading="lazy" src={potentialIcon(potential)} width={14} />}
                            </span>
                            {/* The name in its rarity ink, so a row reads by rarity at a glance. The hover and open fill is accent/40
                                because the inks measure 4.06 to 4.17 on full accent; on accent/40 4.52 to 4.65 (light), 6.86+ dark. */}
                            <span className="w-full truncate font-medium text-[10.5px] leading-tight" style={{ color: rarityInk(operator.rarity) }}>
                                {operator.name}
                            </span>
                            {upgrade && (
                                <span data-slot="recruit-next-potential" title={upgrade.text} className={cn("w-full truncate text-[9.5px] leading-tight tracking-tight", UPGRADE_TONE_CLASS[upgrade.tone])}>
                                    {upgrade.text}
                                </span>
                            )}
                        </button>
                    )}
                />
                <PopoverPopup className="w-max max-w-[min(18rem,calc(100vw-2rem))]">
                    <div className="flex flex-col gap-2 text-left">
                        <div className="flex flex-col gap-0.5">
                            <div className="font-medium text-[13px] text-foreground leading-tight">{operator.name}</div>
                            <div className="flex flex-wrap items-center gap-1.5 text-[11px] text-muted-foreground">
                                <span className="font-mono" style={{ color: rarityInk(operator.rarity) }}>
                                    {getStarsDisplay(operator.rarity)}
                                </span>
                                <span>·</span>
                                <span>{profession}</span>
                                {potentialLabel && (
                                    <>
                                        <span>·</span>
                                        <span>{potentialLabel}</span>
                                    </>
                                )}
                            </div>
                            {upgrade && (
                                <div className="text-[11px] text-muted-foreground">
                                    {t("recruit.result.nextPotential")} <span className={cn("font-medium", UPGRADE_TONE_CLASS[upgrade.tone])}>{upgrade.text}</span>
                                </div>
                            )}
                        </div>
                        <div className="flex flex-col gap-1.5 border-border/60 border-t pt-2">
                            <Kicker>{t("recruit.result.tags")}</Kicker>
                            <OperatorTagList tags={operator.tagList} />
                        </div>
                    </div>
                </PopoverPopup>
            </Popover>
        </li>
    );
}
