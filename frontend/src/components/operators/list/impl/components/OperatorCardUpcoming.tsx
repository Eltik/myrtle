import { Link } from "@tanstack/react-router";
import { useMemo } from "react";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "#/components/ui/preview-card";
import { useDataSaver } from "#/hooks/use-data-saver";
import { useOperatorName } from "#/hooks/use-operator-name";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { getAvatarById, getPortraitById, parseOperatorName } from "#/lib/utils";
import type { IOperatorIndexEntry } from "#/types/operators";
import { RARITY_BLUR_COLORS, RARITY_COLORS } from "../constants";
import { ClassIcon } from "./Icons";
import type { messages } from "./OperatorCardUpcoming.messages";
import { OperatorPreview } from "./OperatorPreview";

export function OperatorCardUpcoming({ operator }: { operator: IOperatorIndexEntry }) {
    const t: TypedT<typeof messages> = useT("operators");
    const name = useOperatorName()(operator);
    const { displayName } = parseOperatorName(name);
    const portrait = getPortraitById(operator.id, "cn");
    const fallback = getAvatarById(operator.id, "cn");
    const dataSaver = useDataSaver();

    // The record is CN's, whose class, archetype and nation names are Chinese; dropping them lets
    // the preview fall back to the same English labels a Global card's preview shows.
    const preview = useMemo(() => ({ ...operator, professionName: "", subProfessionName: "", nationName: null }), [operator]);

    return (
        <HoverCard>
            <HoverCardTrigger
                render={
                    <Link to="/operators/$id" params={{ id: operator.id }} preload={dataSaver ? false : undefined} className="group relative flex aspect-2/3 overflow-clip rounded-md border border-muted/50 bg-card contain-content hover:rounded-lg">
                        <div className={dataSaver ? "absolute inset-x-0 top-0 bottom-12 flex origin-center items-center justify-center p-3 transition-transform group-hover:scale-105" : "absolute inset-0 origin-center transition-transform group-hover:scale-105"}>
                            <img
                                alt={t("card.upcoming.portraitAlt", { name })}
                                className={dataSaver ? "aspect-square w-full max-w-40 rounded-md object-contain" : "h-full w-full rounded-lg object-contain"}
                                decoding="async"
                                height={dataSaver ? 160 : 270}
                                loading="lazy"
                                src={dataSaver ? fallback : portrait}
                                width={dataSaver ? 160 : 180}
                                onError={(e) => {
                                    const img = e.currentTarget;
                                    if (!img.src.endsWith(fallback)) img.src = fallback;
                                }}
                            />
                        </div>
                        <div className="absolute inset-x-0 bottom-0 z-10">
                            <div className="relative">
                                <div className="h-12 w-full bg-background" />
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
                }
            />
            <HoverCardContent className="w-max max-w-85 p-0">
                <OperatorPreview operator={preview} server="cn" />
            </HoverCardContent>
        </HoverCard>
    );
}
