import { Link } from "@tanstack/react-router";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "#/components/ui/preview-card";
import { cn, formatProfession, formatSubProfession, getAvatarById, rarityToNumber } from "#/lib/utils";
import { LIST_GRID_COLS, RARITY_COLORS } from "../constants";
import type { IOperatorView } from "../types";
import { CampIcon, ClassIcon } from "./Icons";
import { OperatorPreview } from "./OperatorPreview";
import { OwnershipBadge } from "./OwnershipBadge";

interface IOperatorCardListProps {
    operator: IOperatorView;
}

function RarityStars({ rarity, className }: { rarity: number; className?: string }) {
    return (
        <span role="img" className={cn("inline-flex items-center gap-0.5", className)} aria-label={`${rarity} star`}>
            {Array.from({ length: rarity }, (_, i) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: decorative stars with fixed count
                <span key={i} aria-hidden="true">
                    ★
                </span>
            ))}
        </span>
    );
}

export function OperatorCardList({ operator }: IOperatorCardListProps) {
    const rarityNum = rarityToNumber(operator.rarity);
    const rarityColor = RARITY_COLORS[rarityNum] ?? "#ffffff";
    const factionLogoId = operator.nationId || operator.teamId || operator.groupId || "rhodes";
    const portraitSrc = getAvatarById(operator.id ?? "");
    const archetype = formatSubProfession(operator.subProfessionId.toLowerCase()).replace(formatProfession(operator.profession), "");

    return (
        <HoverCard>
            <HoverCardTrigger
                render={
                    <Link to="/operators/$id" params={{ id: operator.id ?? "" }} className="group relative block rounded-lg border border-transparent bg-card/50 px-3 py-2.5 transition-colors hover:border-border hover:bg-card">
                        <div className="absolute top-1/2 left-0 h-8 w-0.5 -translate-y-1/2 rounded-full opacity-60 transition-opacity group-hover:opacity-100" style={{ backgroundColor: rarityColor }} />

                        <div className="hidden items-center gap-3 min-[900px]:grid" style={{ gridTemplateColumns: LIST_GRID_COLS }}>
                            <div className="relative h-12 w-12 overflow-hidden rounded-md border border-border/50 bg-background">
                                <img alt={operator.name} src={portraitSrc} loading="lazy" className="absolute inset-0 h-full w-full object-cover transition-transform duration-200 ease-out group-hover:scale-110" />
                            </div>
                            <span className="truncate font-semibold text-foreground text-sm uppercase tracking-wide">{operator.name}</span>
                            <RarityStars rarity={rarityNum} className="justify-self-center" />
                            <div className="flex min-w-0 items-center gap-2 justify-self-center">
                                <ClassIcon profession={operator.profession} size={20} className="opacity-60 transition-opacity group-hover:opacity-100" />
                                <span className="truncate text-muted-foreground text-sm">{formatProfession(operator.profession)}</span>
                            </div>
                            <span className="justify-self-center truncate text-muted-foreground text-sm">{archetype}</span>
                            <div className="justify-self-center">{operator.ownership ? <OwnershipBadge info={operator.ownership} color={rarityColor} /> : <span className="text-muted-foreground/40 text-sm">-</span>}</div>
                            <div className="flex h-6 w-6 items-center justify-center justify-self-center opacity-40 transition-opacity group-hover:opacity-70">
                                <CampIcon groupId={factionLogoId} size={24} />
                            </div>
                        </div>

                        <div className="flex items-center gap-3 min-[900px]:hidden">
                            <div className="relative h-12 w-12 shrink-0 overflow-hidden rounded-md border border-border/50 bg-background">
                                <img alt={operator.name} src={portraitSrc} loading="lazy" className="absolute inset-0 h-full w-full object-cover transition-transform duration-200 ease-out group-hover:scale-110" />
                            </div>
                            <div className="flex min-w-0 flex-1 flex-col gap-1">
                                <div className="flex items-center gap-2">
                                    <span className="truncate font-semibold text-foreground text-sm uppercase tracking-wide">{operator.name}</span>
                                    <div className="flex h-4 w-4 shrink-0 items-center justify-center opacity-50">
                                        <CampIcon groupId={factionLogoId} size={16} />
                                    </div>
                                </div>
                                <div className="flex items-center gap-2 text-muted-foreground text-xs">
                                    <RarityStars rarity={rarityNum} />
                                    <span className="shrink-0 opacity-50">·</span>
                                    <div className="flex min-w-0 items-center gap-1">
                                        <ClassIcon profession={operator.profession} size={14} className="opacity-60" />
                                        <span className="truncate">{formatProfession(operator.profession)}</span>
                                    </div>
                                    <span className="shrink-0 opacity-50">·</span>
                                    <span className="truncate">{archetype}</span>
                                    {operator.ownership && (
                                        <>
                                            <span className="shrink-0 opacity-50">·</span>
                                            <OwnershipBadge info={operator.ownership} color={rarityColor} className="shrink-0" />
                                        </>
                                    )}
                                </div>
                            </div>
                        </div>
                    </Link>
                }
            />
            <HoverCardContent className="w-max max-w-85 p-0">
                <OperatorPreview operator={operator} />
            </HoverCardContent>
        </HoverCard>
    );
}
