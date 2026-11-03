import { Link } from "@tanstack/react-router";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "#/components/ui/preview-card";
import { useDataSaver } from "#/hooks/use-data-saver";
import { assetURL } from "#/lib/assets";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, getAvatarById, rarityToNumber } from "#/lib/utils";
import { RARITY_BLUR_COLORS, RARITY_COLORS } from "../constants";
import type { IOperatorView, StatMetric } from "../types";
import { CnUpdateBadge } from "./CnUpdateBadge";
import { CampIcon, ClassIcon } from "./Icons";
import type { messages } from "./OperatorCardGrid.messages";
import styles from "./OperatorCardGrid.module.css";
import { OperatorPreview } from "./OperatorPreview";
import { OwnershipBadge } from "./OwnershipBadge";

interface IOperatorCardGridProps {
    operator: IOperatorView;
    /** Which community rate the badge shows. Set on /operators and
     *  persisted per viewer; independent of the sort key. */
    statMetric: StatMetric;
}

export function OperatorCardGrid({ operator, statMetric }: IOperatorCardGridProps) {
    const t: TypedT<typeof messages> = useT("operators");
    const dataSaver = useDataSaver();
    const logoId = operator.nationId && operator.nationId.length > 0 ? operator.nationId : operator.teamId && operator.teamId.length > 0 ? operator.teamId : operator.groupId && operator.groupId.length > 0 ? operator.groupId : "rhodes";

    return (
        <HoverCard>
            <HoverCardTrigger
                render={
                    <Link to="/operators/$id" params={{ id: operator.id ?? "" }} preload={dataSaver ? false : undefined} className={cn("group relative flex aspect-2/3 overflow-clip rounded-md border border-muted/50 bg-card contain-content hover:rounded-lg", styles["card-hover-transition"])}>
                        {/* One wrapping row: on a three-column phone grid the two pills side by side ran past the card's edge. */}
                        {operator.ownership || operator.cnChanges ? (
                            <div className="pointer-events-none absolute inset-x-1 top-1 z-20 flex flex-wrap items-start gap-1 [&>*]:pointer-events-auto">
                                {operator.ownership && <OwnershipBadge info={operator.ownership} metric={statMetric} color={RARITY_COLORS[rarityToNumber(operator.rarity)]} />}
                                {operator.cnChanges && <CnUpdateBadge changes={operator.cnChanges} className="ms-auto" />}
                            </div>
                        ) : null}
                        <div className="absolute -translate-x-8 -translate-y-4">
                            <CampIcon groupId={logoId} className="opacity-5 transition-opacity group-hover:opacity-10" size={360} />
                        </div>
                        {dataSaver && operator.id ? (
                            // The deployment-bar avatar, under half the portrait's bytes, sat in the space above the name bar.
                            <div className="absolute inset-x-0 top-0 bottom-12 flex origin-center items-center justify-center p-3 transition-transform group-hover:scale-105">
                                <img alt={t("card.portraitAlt", { name: operator.name })} className="aspect-square w-full max-w-40 rounded-md object-contain" decoding="async" height={160} loading="lazy" src={getAvatarById(operator.id)} width={160} />
                            </div>
                        ) : (
                            <div className="absolute inset-0 origin-center transition-transform group-hover:scale-105">
                                <img alt={t("card.portraitAlt", { name: operator.name })} className="h-full w-full rounded-lg object-contain" decoding="async" height={270} loading="lazy" src={operator.portrait ? assetURL(operator.portrait) : undefined} width={180} />
                            </div>
                        )}
                        <div className="absolute inset-x-0 bottom-0 z-10">
                            <div className="relative">
                                {/* Solid: a blur per card cost up to 100 backdrop passes a page, and the name never needed the art behind it. */}
                                <div className="h-12 w-full bg-background" />
                                {/* Ends short of the class icon (16px, 24px from md) so "Leizi the Thunderbringer" wraps instead of running under it; two tight lines fit the 48px bar. */}
                                <h2 className="absolute right-5 bottom-1 left-1 line-clamp-2 font-bold text-xs uppercase leading-tight opacity-60 opacity-transition group-hover:opacity-100 sm:text-sm md:right-7 md:text-sm">{operator.name}</h2>
                                <div className="card-hover-transition absolute right-1 bottom-1 flex scale-75 items-center opacity-90 transition-transform group-hover:scale-100 group-hover:opacity-100">
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
                }
            />
            <HoverCardContent className="w-max max-w-85 p-0">
                <OperatorPreview operator={operator} ownership={operator.ownership} cnChanges={operator.cnChanges} />
            </HoverCardContent>
        </HoverCard>
    );
}
