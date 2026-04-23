import { HoverCard, HoverCardContent, HoverCardTrigger } from "#/components/ui/preview-card";
import type { IOperatorListItem } from "#/types/operators";
import { Link } from "@tanstack/react-router";
import styles from "./OperatorCardGrid.module.css";
import { cn, rarityToNumber } from "#/lib/utils";
import { CampIcon, ClassIcon } from "./Icons";
import { env } from "#/env";
import { RARITY_BLUR_COLORS, RARITY_COLORS } from "../constants";
import { OperatorPreview } from "./OperatorPreview";

interface IOperatorCardGridProps {
    operator: IOperatorListItem;
}

export function OperatorCardGrid({ operator }: IOperatorCardGridProps) {
    const logoId = operator.nationId && operator.nationId.length > 0 ? operator.nationId : operator.teamId && operator.teamId.length > 0 ? operator.teamId : operator.groupId && operator.groupId.length > 0 ? operator.groupId : "rhodes";

    return (
        <HoverCard>
            <HoverCardTrigger>
                <Link to={`/operators/${operator.id}`} className={cn("group relative flex aspect-2/3 overflow-clip rounded-md border border-muted/50 bg-card contain-content hover:rounded-lg", styles["card-hover-transition"])}>
                    <div className="absolute -translate-x-8 -translate-y-4">
                        <CampIcon groupId={logoId} className="opacity-5 transition-opacity group-hover:opacity-10" size={360} />
                    </div>
                    <div className="absolute inset-0 origin-center transition-transform group-hover:scale-105">
                        <img alt={`${operator.name} portrait`} className="h-full w-full rounded-lg object-contain" decoding="async" loading="lazy" src={`${env.VITE_BACKEND_URL}/api/assets${operator.portrait}`} />
                    </div>
                    <div className="absolute inset-x-0 bottom-0 z-10">
                        <div className="relative">
                            <div className="h-12 w-full bg-background/80 backdrop-blur-sm" />
                            <h2 className="absolute bottom-1 left-1 line-clamp-2 max-w-[92%] font-bold text-xs uppercase opacity-60 opacity-transition group-hover:opacity-100 sm:text-sm md:text-sm">{operator.name}</h2>
                            <div className="card-hover-transition absolute right-1 bottom-1 flex scale-75 items-center opacity-90 group-hover:scale-100 group-hover:opacity-100 transition-transform">
                                <div className="h-4 w-4 md:h-6 md:w-6">
                                    <ClassIcon profession={operator.profession} size={160} />
                                </div>
                            </div>
                            <div
                                className="absolute bottom-0 h-0.5 w-full"
                                style={{
                                    backgroundColor: RARITY_COLORS[rarityToNumber(operator.rarity)],
                                }}
                            />
                            <div
                                className="absolute -bottom-0.5 h-1 w-full blur-sm"
                                style={{
                                    backgroundColor: RARITY_BLUR_COLORS[rarityToNumber(operator.rarity)],
                                }}
                            />
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
