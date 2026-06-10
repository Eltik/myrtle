import { Link } from "@tanstack/react-router";
import { getAvatarById, getPortraitById, parseOperatorName } from "#/lib/utils";
import type { IOperatorIndexEntry } from "#/types/operators";
import { RARITY_BLUR_COLORS, RARITY_COLORS } from "../constants";
import { ClassIcon } from "./Icons";

export function OperatorCardUpcoming({ operator }: { operator: IOperatorIndexEntry }) {
    const { displayName } = parseOperatorName(operator.name);
    const portrait = getPortraitById(operator.id, "cn");
    const fallback = getAvatarById(operator.id, "cn");

    return (
        <Link to="/operators/$id" params={{ id: operator.id }} className="group relative flex aspect-2/3 overflow-clip rounded-md border border-muted/50 bg-card contain-content hover:rounded-lg">
            <div className="absolute inset-0 origin-center transition-transform group-hover:scale-105">
                <img
                    alt={`${operator.name} portrait`}
                    className="h-full w-full rounded-lg object-contain"
                    decoding="async"
                    loading="lazy"
                    src={portrait}
                    onError={(e) => {
                        const img = e.currentTarget;
                        if (!img.src.endsWith(fallback)) img.src = fallback;
                    }}
                />
            </div>
            <div className="absolute inset-x-0 bottom-0 z-10">
                <div className="relative">
                    <div className="h-12 w-full bg-background/80 backdrop-blur-sm" />
                    <h2 className="absolute bottom-1 left-1 line-clamp-2 max-w-[92%] font-bold text-xs uppercase opacity-80">{displayName}</h2>
                    <div className="absolute right-1 bottom-1 flex items-center opacity-90">
                        <div className="h-4 w-4 md:h-6 md:w-6">
                            <ClassIcon profession={operator.profession} size={160} />
                        </div>
                    </div>
                    <div className="absolute bottom-0 h-0.5 w-full" style={{ backgroundColor: RARITY_COLORS[operator.rarity] }} />
                    <div className="absolute -bottom-0.5 h-1 w-full blur-sm" style={{ backgroundColor: RARITY_BLUR_COLORS[operator.rarity] }} />
                </div>
            </div>
        </Link>
    );
}
