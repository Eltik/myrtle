import { Link } from "@tanstack/react-router";
import { ChevronRight } from "lucide-react";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { Badge } from "#/components/ui/badge";
import { Card } from "#/components/ui/card";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { getAvatarById } from "#/lib/utils";
import { DEFAULT_AVATAR_ID } from "../constants";
import type { DisplayUser } from "../types";
import type { messages } from "./UserCard.messages";

export function UserCard({ user }: { user: DisplayUser }) {
    const t: TypedT<typeof messages> = useT("user");
    const f = useFormatters();
    const nickname = user.nickname ?? `Doctor ${user.uid}`;
    const initials = (user.nickname ?? user.uid).slice(0, 2).toUpperCase();
    const avatarSrc = getAvatarById(user.avatar_id ?? DEFAULT_AVATAR_ID);

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
                        {user.operator_count != null && (
                            <span>
                                <span className="font-medium text-foreground">{f.number(user.operator_count)}</span> {t("search.card.operators")}
                            </span>
                        )}
                        {user.skin_count != null && (
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
