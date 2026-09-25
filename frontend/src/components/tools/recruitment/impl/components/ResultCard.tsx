import type * as React from "react";
import { potentialIcon } from "#/components/operators/detail/impl/assets";
import { Badge } from "#/components/ui/badge";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Popover, PopoverPopup, PopoverTrigger } from "#/components/ui/popover";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, RARITY_HEX_MUTED } from "#/lib/utils";
import { guaranteedFloorRarity } from "../calculator";
import { PROFESSION_LABELS, RARITY_COLORS } from "../constants";
import { getStarsDisplay } from "../helpers";
import type { IRecruitableOperator, IRosterOverlay, ITagCombinationResult, ResultLayout } from "../types";
import type { messages } from "./ResultCard.messages";

export type ResultT = TypedT<typeof messages>;

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
                {result.tagNames.map((name) => (
                    <Badge key={name} variant="outline" size="default">
                        {name}
                    </Badge>
                ))}
                <span className="basis-full max-sm:hidden" aria-hidden="true" />
                <GuaranteedBadge result={result} layout="compact" />
            </div>
            <ul className="flex min-w-0 flex-wrap content-start gap-1">
                {result.operators.map((op) => (
                    <OperatorTile key={op.id} operator={op} roster={roster} />
                ))}
            </ul>
        </div>
    );
}

/** Rarity 1 is the robot tier, and "★" alone would read as an ordinary 1★. */
function floorLabel(rarity: number, t: ResultT): string {
    return rarity === 1 ? t("recruit.result.robotFloor") : t("recruit.result.starFloor", { rarity });
}

/**
 * The guaranteed worst case, shown on every card because it is the key the list
 * is ranked by - leaving it off the cards below a 5★ lock meant the ordering was
 * driven by a value the reader could not see.
 *
 * A 5★/6★ lock keeps the filled badge it always had: that is a genuinely
 * different thing, a combination that cannot miss. Everything else gets a quiet
 * label - no fill, no border, colour only on the value - so the floor is legible
 * at a glance without competing with the lock.
 *
 * `layout` only changes the alignment: the compact row pushes the badge right
 * on phones, the detailed card header pushes the quiet label right at every width.
 */
export function GuaranteedBadge({ result, layout }: { result: ITagCombinationResult; layout: ResultLayout }): React.ReactElement {
    const t: ResultT = useT("tools");
    const floor = guaranteedFloorRarity(result);
    const colors = RARITY_COLORS[floor];

    if (floor >= 5) {
        return <span className={cn("inline-flex h-5.5 shrink-0 items-center whitespace-nowrap rounded-sm border px-1.5 font-medium text-sm sm:h-4.5 sm:text-xs", layout === "compact" && "max-sm:ml-auto", colors?.border, colors?.bg, colors?.text)}>{t("recruit.result.guaranteed", { rarity: floor })}</span>;
    }

    return (
        <span className={cn("inline-flex h-5.5 shrink-0 items-center gap-1 whitespace-nowrap px-0.5 font-medium text-xs sm:h-4.5 sm:text-[11px]", layout === "compact" ? "max-sm:ml-auto" : "ml-auto")} title={t("recruit.result.minTitle", { floor: floorLabel(floor, t) })}>
            <span className="text-muted-foreground">{t("recruit.result.min")}</span>
            <span className={cn("font-mono", colors?.text)}>{floorLabel(floor, t)}</span>
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
    gain: "text-foreground/80",
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
    const colors = RARITY_COLORS[operator.rarity];
    const profession = PROFESSION_LABELS[operator.profession] ?? operator.profession;

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
                        <button {...props} type="button" className="flex w-full cursor-pointer flex-col items-center gap-0.5 rounded-md p-0.5 text-center transition-colors hover:bg-accent focus-visible:outline-2 focus-visible:outline-ring data-popup-open:bg-accent">
                            <span className="relative block size-11 shrink-0 overflow-hidden rounded-md font-semibold text-[13px] leading-11" style={{ backgroundColor: `${hex}33`, boxShadow: `inset 0 0 0 1px ${hex}73` }}>
                                <span aria-hidden="true" className={cn("block size-full", showPotential && !owned && "opacity-50 grayscale")}>
                                    <OperatorAvatar charId={operator.id} name={operator.name} />
                                </span>
                                <span aria-hidden="true" className="absolute inset-x-0 bottom-0 h-[3px]" style={{ backgroundColor: hex }} />
                                {potentialLabel && potential !== undefined && potential > 0 && <img alt={potentialLabel} className="icon-theme-aware absolute bottom-0.5 left-0 h-4 w-3.5 object-contain drop-shadow-sm" decoding="async" height={16} loading="lazy" src={potentialIcon(potential)} width={14} />}
                            </span>
                            <span className="w-full truncate font-medium text-[10.5px] text-foreground leading-tight">{operator.name}</span>
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
                                <span className={cn("font-mono", colors?.text)}>{getStarsDisplay(operator.rarity)}</span>
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
                            <div className="font-medium text-[10.5px] text-muted-foreground uppercase tracking-wider">{t("recruit.result.tags")}</div>
                            <OperatorTagList tags={operator.tagList} />
                        </div>
                    </div>
                </PopoverPopup>
            </Popover>
        </li>
    );
}
