import { HoverCard, HoverCardContent, HoverCardTrigger } from "#/components/ui/preview-card";
import { env } from "#/env";
import { capitalize, cn, formatProfession, formatSubProfession, rarityToNumber } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";
import { Link } from "@tanstack/react-router";
import { LIST_GRID_COLS, RARITY_COLORS } from "../constants";
import { CampIcon, ClassIcon } from "./Icons";
import { OperatorPreview } from "./OperatorPreview";

interface IOperatorCardListProps {
    operator: IOperatorListItem;
}

function RarityStars({ rarity, className }: { rarity: number; className?: string }) {
    return (
        <span className={cn("inline-flex items-center gap-0.5", className)} aria-label={`${rarity} star`}>
            {Array.from({ length: rarity }, (_, i) => (
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
    const portraitSrc = `${env.VITE_BACKEND_URL}/api/avatar/${operator.id}`;
    const archetype = formatSubProfession(operator.subProfessionId.toLowerCase()).replace(formatProfession(operator.profession), "");

    return (
        <HoverCard>
            <HoverCardTrigger>
                <Link to={`/operators/${operator.id}`} className="group relative block rounded-lg border border-transparent bg-card/50 px-3 py-2.5 transition-colors hover:border-border hover:bg-card">
                    <div className="absolute left-0 top-1/2 h-8 w-0.5 -translate-y-1/2 rounded-full opacity-60 transition-opacity group-hover:opacity-100" style={{ backgroundColor: rarityColor }} />

                    <div className="hidden items-center gap-3 min-[900px]:grid" style={{ gridTemplateColumns: LIST_GRID_COLS }}>
                        <div className="relative h-12 w-12 overflow-hidden rounded-md border border-border/50 bg-background">
                            <img alt={operator.name} src={portraitSrc} loading="lazy" className="absolute inset-0 h-full w-full object-cover transition-transform duration-200 ease-out group-hover:scale-110" />
                        </div>
                        <span className="truncate text-sm font-semibold uppercase tracking-wide text-foreground">{operator.name}</span>
                        <RarityStars rarity={rarityNum} className="justify-self-center" />
                        <div className="flex min-w-0 items-center gap-2 justify-self-center">
                            <ClassIcon profession={operator.profession} size={20} className="opacity-60 transition-opacity group-hover:opacity-100" />
                            <span className="truncate text-sm text-muted-foreground">{formatProfession(operator.profession)}</span>
                        </div>
                        <span className="truncate text-sm text-muted-foreground justify-self-center">{archetype}</span>
                        <div className="flex h-6 w-6 items-center justify-center opacity-40 transition-opacity group-hover:opacity-70 justify-self-center">
                            <CampIcon groupId={factionLogoId} size={24} />
                        </div>
                    </div>

                    <div className="flex items-center gap-3 min-[900px]:hidden">
                        <div className="relative h-12 w-12 shrink-0 overflow-hidden rounded-md border border-border/50 bg-background">
                            <img alt={operator.name} src={portraitSrc} loading="lazy" className="absolute inset-0 h-full w-full object-cover transition-transform duration-200 ease-out group-hover:scale-110" />
                        </div>
                        <div className="flex min-w-0 flex-1 flex-col gap-1">
                            <div className="flex items-center gap-2">
                                <span className="truncate text-sm font-semibold uppercase tracking-wide text-foreground">{operator.name}</span>
                                <div className="flex h-4 w-4 shrink-0 items-center justify-center opacity-50">
                                    <CampIcon groupId={factionLogoId} size={16} />
                                </div>
                            </div>
                            <div className="flex items-center gap-2 text-xs text-muted-foreground">
                                <RarityStars rarity={rarityNum} />
                                <span className="shrink-0 opacity-50">·</span>
                                <div className="flex min-w-0 items-center gap-1">
                                    <ClassIcon profession={operator.profession} size={14} className="opacity-60" />
                                    <span className="truncate">{formatProfession(operator.profession)}</span>
                                </div>
                                <span className="shrink-0 opacity-50">·</span>
                                <span className="truncate">{archetype}</span>
                            </div>
                        </div>
                    </div>
                </Link>
            </HoverCardTrigger>
            <HoverCardContent className="w-max max-w-85 p-0">
                <OperatorPreview operator={operator} />
            </HoverCardContent>
        </HoverCard>
    );
}
