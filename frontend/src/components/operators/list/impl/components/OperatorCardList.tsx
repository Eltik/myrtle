import { Link } from "@tanstack/react-router";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "#/components/ui/preview-card";
import { useDataSaver } from "#/hooks/use-data-saver";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { archetypeLabel, cn, getAvatarById, professionLabel, rarityToNumber } from "#/lib/utils";
import { LIST_GRID_COLS, RARITY_COLORS } from "../constants";
import type { IOperatorView, StatMetric } from "../types";
import { CnUpdateBadge } from "./CnUpdateBadge";
import { CampIcon, ClassIcon } from "./Icons";
import type { messages } from "./OperatorCardList.messages";
import { OperatorPreview } from "./OperatorPreview";
import { OwnershipBadge } from "./OwnershipBadge";

interface IOperatorCardListProps {
    operator: IOperatorView;
    /** Which community rate the badge shows. Set on /operators and
     *  persisted per viewer; independent of the sort key. */
    statMetric: StatMetric;
}

function RarityStars({ rarity, className }: { rarity: number; className?: string }) {
    const t: TypedT<typeof messages> = useT("operators");

    return (
        <span role="img" className={cn("inline-flex items-center gap-0.5", className)} aria-label={t("card.rarityStars", { rarity })}>
            {Array.from({ length: rarity }, (_, i) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: decorative stars with fixed count
                <span key={i} aria-hidden="true">
                    ★
                </span>
            ))}
        </span>
    );
}

export function OperatorCardList({ operator, statMetric }: IOperatorCardListProps) {
    const t: TypedT<typeof messages> = useT("operators");
    const rarityNum = rarityToNumber(operator.rarity);
    const rarityColor = RARITY_COLORS[rarityNum] ?? "#ffffff";
    const factionLogoId = operator.nationId || operator.teamId || operator.groupId || "rhodes";
    const portraitSrc = getAvatarById(operator.id ?? "");
    const dataSaver = useDataSaver();
    const archetype = archetypeLabel(operator);

    return (
        <HoverCard>
            <HoverCardTrigger
                render={
                    <Link to="/operators/$id" params={{ id: operator.id ?? "" }} preload={dataSaver ? false : undefined} className="group relative block rounded-lg border border-transparent bg-card/50 px-3 py-2.5 transition-colors hover:border-border hover:bg-card">
                        <div className="absolute top-1/2 left-0 h-8 w-0.5 -translate-y-1/2 rounded-full opacity-60 transition-opacity group-hover:opacity-100" style={{ backgroundColor: rarityColor }} />

                        <div className="hidden items-center gap-3 min-[900px]:grid" style={{ gridTemplateColumns: LIST_GRID_COLS }}>
                            <div className="relative h-12 w-12 overflow-hidden rounded-md border border-border/50 bg-background">
                                <img alt={operator.name} src={portraitSrc} loading="lazy" width={48} height={48} className="absolute inset-0 h-full w-full object-cover transition-transform duration-200 ease-out group-hover:scale-110" />
                            </div>
                            <div className="flex min-w-0 items-center gap-2">
                                <span className="truncate font-semibold text-foreground text-sm uppercase tracking-wide">{operator.name}</span>
                                {operator.cnChanges && <CnUpdateBadge changes={operator.cnChanges} className="shrink-0" />}
                            </div>
                            <RarityStars rarity={rarityNum} className="justify-self-center" />
                            <div className="flex min-w-0 items-center gap-2 justify-self-center">
                                <ClassIcon profession={operator.profession} size={20} className="opacity-60 transition-opacity group-hover:opacity-100" />
                                <span className="truncate text-muted-foreground text-sm">{professionLabel(operator)}</span>
                            </div>
                            <span className="justify-self-center truncate text-muted-foreground text-sm">{archetype}</span>
                            <div className="justify-self-center">
                                {operator.ownership ? (
                                    <OwnershipBadge info={operator.ownership} metric={statMetric} color={rarityColor} />
                                ) : (
                                    <span className="text-muted-foreground text-sm">
                                        <span aria-hidden="true">-</span>
                                        <span className="sr-only">{t("card.noOwnership")}</span>
                                    </span>
                                )}
                            </div>
                            <div className="flex h-6 w-6 items-center justify-center justify-self-center overflow-hidden opacity-40 transition-opacity group-hover:opacity-70">
                                <CampIcon groupId={factionLogoId} size={24} />
                            </div>
                        </div>

                        <div className="flex items-center gap-3 min-[900px]:hidden">
                            <div className="relative h-12 w-12 shrink-0 overflow-hidden rounded-md border border-border/50 bg-background">
                                <img alt={operator.name} src={portraitSrc} loading="lazy" width={48} height={48} className="absolute inset-0 h-full w-full object-cover transition-transform duration-200 ease-out group-hover:scale-110" />
                            </div>
                            <div className="flex min-w-0 flex-1 flex-col gap-1">
                                <div className="flex items-center gap-2">
                                    <span className="truncate font-semibold text-foreground text-sm uppercase tracking-wide">{operator.name}</span>
                                    {operator.cnChanges && <CnUpdateBadge changes={operator.cnChanges} className="shrink-0" />}
                                    <div className="flex h-4 w-4 shrink-0 items-center justify-center overflow-hidden opacity-50">
                                        <CampIcon groupId={factionLogoId} size={16} />
                                    </div>
                                </div>
                                {/* At 375 class, subclass and the share chip in one line truncated both names to
                                    "Van... Ch...". The class reads from its icon here; the line wraps, so the chip
                                    drops below a long subclass instead of squeezing it. */}
                                <div className="flex flex-wrap items-center gap-x-2 gap-y-1 text-muted-foreground text-xs">
                                    <RarityStars rarity={rarityNum} />
                                    <span aria-hidden="true" className="shrink-0">
                                        ·
                                    </span>
                                    <div className="flex min-w-0 items-center gap-1">
                                        <ClassIcon profession={operator.profession} size={14} className="shrink-0 opacity-60" />
                                        <span className="truncate">{archetype}</span>
                                    </div>
                                    {operator.ownership && (
                                        <>
                                            <span aria-hidden="true" className="shrink-0">
                                                ·
                                            </span>
                                            <OwnershipBadge info={operator.ownership} metric={statMetric} color={rarityColor} className="shrink-0" />
                                        </>
                                    )}
                                </div>
                            </div>
                        </div>
                    </Link>
                }
            />
            <HoverCardContent className="w-max max-w-85 p-0">
                <OperatorPreview operator={operator} ownership={operator.ownership} cnChanges={operator.cnChanges} />
            </HoverCardContent>
        </HoverCard>
    );
}
