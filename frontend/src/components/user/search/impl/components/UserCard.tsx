import { Link } from "@tanstack/react-router";
import { ChevronRight } from "lucide-react";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { Badge } from "#/components/ui/badge";
import { Card } from "#/components/ui/card";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { formatArchetype, formatProfession, getAvatarById } from "#/lib/utils";
import { DEFAULT_AVATAR_ID } from "../constants";
import { DEFAULT_SORT, parseScope } from "../searchControls";
import type { DisplayUser } from "../types";
import type { messages } from "./UserCard.messages";

interface IUserCardProps {
    user: DisplayUser;
    /** The active `sort` token; under anything but `score` the row's `metric` leads the stats. */
    sort?: string;
}

export function UserCard({ user, sort = DEFAULT_SORT }: IUserCardProps) {
    const t: TypedT<typeof messages> = useT("user");
    const f = useFormatters();
    const nickname = user.nickname ?? `Player ${user.uid}`;
    const initials = (user.nickname ?? user.uid).slice(0, 2).toUpperCase();
    const avatarSrc = getAvatarById(user.avatar_id ?? DEFAULT_AVATAR_ID);
    const metric = sort !== DEFAULT_SORT && user.metric != null ? formatMetric(sort, user.metric, t, f.date) : null;

    return (
        <Card className="group transition-shadow duration-150 hover:shadow-md">
            <Link to="/user/$id" params={{ id: user.uid }} className="flex items-center gap-3.5 px-4 py-3.5 no-underline">
                <Avatar className="h-14 w-14 shrink-0 rounded-xl border border-border transition-transform duration-200 group-hover:scale-105">
                    <AvatarImage src={avatarSrc} alt={nickname} />
                    <AvatarFallback className="rounded-xl text-sm">{initials}</AvatarFallback>
                </Avatar>

                <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-1.5 truncate">
                        <span className="truncate font-sans font-semibold text-[14px] text-foreground leading-snug transition-colors duration-150 group-hover:text-primary">{nickname}</span>
                        {user.grade && (
                            <Badge variant="outline" size="sm" className="font-mono">
                                {user.grade}
                            </Badge>
                        )}
                    </div>

                    <div className="mt-1 flex flex-wrap items-center gap-x-2 gap-y-0.5">
                        <span className="font-mono text-[11px] text-muted-foreground tabular-nums leading-none">{user.uid}</span>
                        <Badge variant="secondary" size="sm" className="font-mono uppercase">
                            {user.server}
                        </Badge>
                    </div>

                    <div className="mt-1.5 flex items-center gap-2.5 font-sans text-[11.5px] text-muted-foreground leading-none">
                        {metric != null && <span className="font-semibold text-primary">{metric}</span>}
                        {user.level != null && (
                            <span>
                                <span className="font-medium text-foreground">{t("search.card.level", { level: user.level })}</span>
                            </span>
                        )}
                        {user.total_score != null && (
                            <span>
                                <span className="font-medium text-foreground">{f.number(user.total_score)}</span> {t("search.card.points")}
                            </span>
                        )}
                        {user.operator_count != null && sort !== "operators" && (
                            <span>
                                <span className="font-medium text-foreground">{f.number(user.operator_count)}</span> {t("search.card.operators")}
                            </span>
                        )}
                        {user.skin_count != null && sort !== "skins" && (
                            <span>
                                <span className="font-medium text-foreground">{f.number(user.skin_count)}</span> {t("search.card.skins")}
                            </span>
                        )}
                    </div>
                </div>

                <ChevronRight className="h-3.5 w-3.5 shrink-0 text-muted-foreground/50 transition-transform duration-150 group-hover:translate-x-0.5" aria-hidden="true" />
            </Link>
        </Card>
    );
}

/** The row's metric worded for its sort; `joined` carries a unix timestamp in seconds. */
function formatMetric(sort: string, metric: number, t: TypedT<typeof messages>, date: (value: number) => string): string {
    const scope = parseScope(sort);
    if (scope) return t("search.card.metric.scoped", { count: metric, label: scope.kind === "class" ? formatProfession(scope.profession) : formatArchetype(scope.subProfessionId) });
    switch (sort) {
        case "operators":
            return t("search.card.metric.operators", { count: metric });
        case "joined":
            return t("search.card.metric.joined", { date: date(metric * 1000) });
        case "enemies":
            return t("search.card.metric.enemies", { count: metric });
        case "potentials":
            return t("search.card.metric.potentials", { count: metric });
        case "masteries":
            return t("search.card.metric.masteries", { count: metric });
        case "modules":
            return t("search.card.metric.modules", { count: metric });
        case "skins":
            return t("search.card.metric.skins", { count: metric });
        default:
            return String(metric);
    }
}
