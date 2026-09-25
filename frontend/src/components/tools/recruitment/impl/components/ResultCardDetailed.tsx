import { ChevronDown } from "lucide-react";
import type * as React from "react";
import { useState } from "react";
import { potentialIcon } from "#/components/operators/detail/impl/assets";
import { Badge } from "#/components/ui/badge";
import { Card, CardHeader, CardPanel } from "#/components/ui/card";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "#/components/ui/preview-card";
import { useT } from "#/lib/i18n";
import { cn } from "#/lib/utils";
import { PROFESSION_LABELS, RARITY_COLORS } from "../constants";
import { getStarsDisplay } from "../helpers";
import type { IRecruitableOperator, IRosterOverlay, ITagCombinationResult } from "../types";
import { GuaranteedBadge, nextUpgradeLabel, OperatorTagList, type ResultT, UPGRADE_TONE_CLASS } from "./ResultCard";

interface IResultCardDetailedProps {
    result: ITagCombinationResult;
    roster: IRosterOverlay | null;
}

/**
 * The "Detailed" layout: one card per tag combination, the tags and the
 * guaranteed floor in its header, each operator a wide row (portrait, name,
 * stars, class, next potential) three to a line. The tags an operator carries
 * open on hover for a pointer and expand under the row on a phone.
 */
export function ResultCardDetailed({ result, roster }: IResultCardDetailedProps): React.ReactElement {
    return (
        <Card>
            <CardHeader className="flex flex-row flex-wrap items-center justify-between gap-x-3 gap-y-1.5 px-3 py-2.5 sm:px-4 sm:py-3">
                <div className="flex min-w-0 flex-1 flex-wrap items-center gap-1.5">
                    {result.tagNames.map((name) => (
                        <Badge key={name} variant="outline" size="default">
                            {name}
                        </Badge>
                    ))}
                </div>
                <GuaranteedBadge result={result} layout="detailed" />
            </CardHeader>
            <CardPanel className="px-3 pt-0 pb-3 sm:px-4">
                <ul className="grid grid-cols-1 gap-1.5 sm:grid-cols-2 lg:grid-cols-3">
                    {result.operators.map((op) => (
                        <OperatorRow key={op.id} operator={op} roster={roster} />
                    ))}
                </ul>
            </CardPanel>
        </Card>
    );
}

function OperatorRow({ operator, roster }: { operator: IRecruitableOperator; roster: IRosterOverlay | null }): React.ReactElement {
    const t: ResultT = useT("tools");
    const colors = RARITY_COLORS[operator.rarity];
    const profession = PROFESSION_LABELS[operator.profession] ?? operator.profession;
    const [mobileExpanded, setMobileExpanded] = useState(false);

    const potential = roster?.potentialByOperator.get(operator.id);
    const owned = potential !== undefined;
    const showPotential = roster?.showPotentials ?? false;
    const upgrade = roster?.showNextUpgrade ? nextUpgradeLabel(operator, potential, t) : null;

    return (
        <li className={cn("rounded-md border transition-colors", colors?.border, colors?.bg, colors?.hoverBg, colors?.hoverBorder)}>
            <HoverCard>
                <HoverCardTrigger
                    render={
                        <button type="button" onClick={() => setMobileExpanded((v) => !v)} aria-expanded={mobileExpanded} aria-controls={`op-tags-${operator.id}`} className="flex w-full cursor-pointer items-center gap-2 rounded-md px-2 py-2 text-left sm:py-1.5">
                            <span className="relative inline-flex size-8 shrink-0 items-center justify-center rounded-md bg-muted font-semibold text-[11px]">
                                <span aria-hidden="true" className={cn("inline-flex size-8 items-center justify-center overflow-hidden rounded-md", showPotential && !owned && "opacity-50 grayscale")}>
                                    <OperatorAvatar charId={operator.id} name={operator.name} />
                                </span>
                                {showPotential && owned && potential > 0 && (
                                    <img alt={t("recruit.result.potentialAlt", { rank: potential + 1 })} className="icon-theme-aware absolute -bottom-1 -left-1 h-4 w-3.5 object-contain drop-shadow-sm" decoding="async" height={16} loading="lazy" src={potentialIcon(potential)} width={14} />
                                )}
                            </span>
                            <div className="min-w-0 flex-1">
                                <div className="truncate font-medium text-[13px] text-foreground leading-tight">{operator.name}</div>
                                <div className="flex items-center gap-1.5 text-[11px] text-muted-foreground">
                                    <span className={cn("font-mono", colors?.text)}>{getStarsDisplay(operator.rarity)}</span>
                                    <span>·</span>
                                    <span className="truncate">{profession}</span>
                                    {upgrade && (
                                        <>
                                            <span>·</span>
                                            <span data-slot="recruit-next-potential" className={cn("truncate font-mono", UPGRADE_TONE_CLASS[upgrade.tone])}>
                                                {upgrade.text}
                                            </span>
                                        </>
                                    )}
                                </div>
                            </div>
                            <ChevronDown aria-hidden="true" className={cn("size-3.5 shrink-0 text-muted-foreground transition-transform duration-200 sm:hidden", mobileExpanded && "rotate-180")} />
                        </button>
                    }
                />
                <HoverCardContent className="hidden w-max max-w-72 p-3 sm:flex">
                    <div className="flex flex-col gap-2">
                        <div className="font-medium text-[12px] text-muted-foreground">{t("recruit.result.tags")}</div>
                        <OperatorTagList tags={operator.tagList} />
                    </div>
                </HoverCardContent>
            </HoverCard>
            {mobileExpanded && (
                <div id={`op-tags-${operator.id}`} className="flex flex-col gap-1.5 border-border/40 border-t px-2 py-2 sm:hidden">
                    <div className="font-medium text-[10.5px] text-muted-foreground uppercase tracking-wider">{t("recruit.result.tags")}</div>
                    <OperatorTagList tags={operator.tagList} />
                </div>
            )}
        </li>
    );
}
