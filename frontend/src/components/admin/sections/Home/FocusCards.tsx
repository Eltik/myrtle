import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useMemo } from "react";
import { useLevelLabel } from "#/components/admin/sections/Users/labels";
import type { IAdminAccess } from "#/components/admin/shell/access";
import { useAdminLocales } from "#/components/admin/shell/locales";
import { useEmptyOperators } from "#/components/admin/shell/useInboxQueue";
import { Button } from "#/components/ui/button";
import { Card, CardDescription, CardHeader, CardPanel, CardTitle } from "#/components/ui/card";
import { Progress } from "#/components/ui/progress";
import { useAuth } from "#/hooks/use-auth";
import { translationProgressQueryOptions } from "#/lib/api/admin";
import { browseTierListsQueryOptions, grantedTierListsQueryOptions } from "#/lib/api/tier-lists";
import { useFormatters, useGamedataServer, useT } from "#/lib/i18n";
import { buildFocusCards, type FocusCard, type IFocusTierList } from "./queue";
import { type HomeT, namesSub } from "./useHomeLabels";

interface ICardView {
    key: string;
    area: string;
    title: string;
    sub: string | null;
    progress: { value: number; label: string } | null;
    cta: string;
    variant: "default" | "outline";
    go: () => void;
}

/** What a translator or tier list editor opens Home to find (design `focusCards`). */
export function FocusCards({ access }: { access: IAdminAccess }): React.ReactElement {
    const t: HomeT = useT("admin");
    const f = useFormatters();
    const navigate = useNavigate();
    const levelLabel = useLevelLabel();
    const { isAuthenticated } = useAuth();
    const isTranslator = access.role === "translator";
    const isEditor = access.role === "tier_list_editor";

    const progressQuery = useQuery({ ...translationProgressQueryOptions(isAuthenticated), enabled: isAuthenticated && isTranslator && access.myLoc.length > 0 });
    const locales = useAdminLocales(isTranslator);
    const empty = useEmptyOperators(isAuthenticated && isEditor);
    const grantedQuery = useQuery({ ...grantedTierListsQueryOptions(isAuthenticated), enabled: isAuthenticated && isEditor });
    const browseQuery = useQuery({ ...browseTierListsQueryOptions(useGamedataServer()), enabled: isEditor && (grantedQuery.data?.length ?? 0) > 0 });

    const tierLists = useMemo((): IFocusTierList[] => {
        return (grantedQuery.data ?? []).map((g) => {
            const listed = browseQuery.data?.find((b) => b.slug === g.slug);
            return {
                slug: g.slug,
                title: g.title,
                permission: g.permission,
                tiers: listed?.tiers.length,
                placements: listed?.tiers.reduce((sum, tier) => sum + tier.operators.length, 0),
                updatedAtMs: listed?.updatedAtMs,
            };
        });
    }, [grantedQuery.data, browseQuery.data]);

    // The translator's languages, in the locale table's order.
    const localeGrants = useMemo(() => {
        const order = new Map(locales.list.map((l, i) => [l.code, i]));
        return [...access.myLoc].sort((a, b) => (order.get(a.code) ?? 999) - (order.get(b.code) ?? 999));
    }, [access.myLoc, locales.list]);

    const loading = access.grantsLoading || (isTranslator && access.myLoc.length > 0 && (!progressQuery.data || !locales.loaded)) || (isEditor && (!empty || !grantedQuery.data));

    if (loading) return <div className="rounded-2xl border border-border border-dashed p-7 text-[14px] text-muted-foreground">{t("home.focus.loading")}</div>;

    const cards = buildFocusCards({ role: access.role, localeGrants, progress: progressQuery.data ?? [], emptyOps: empty?.empty ?? [], totalOps: empty?.total ?? 0, tierLists });

    const toView = (card: FocusCard): ICardView => {
        switch (card.kind) {
            case "locale": {
                const loc = locales.list.find((l) => l.code === card.code);
                const todo = card.stale + card.missing > 0;
                return {
                    key: `locale-${card.code}`,
                    area: card.editable ? t("home.focus.locale.area", { level: levelLabel(card.level) }) : t("home.focus.locale.areaViewOnly"),
                    title: loc ? `${loc.englishName} ${loc.nativeName}` : card.code,
                    sub: todo ? t("home.focus.locale.sub", { stale: f.number(card.stale), missing: f.number(card.missing) }) : t("home.focus.locale.complete"),
                    progress: { value: card.percent, label: f.percent(card.percent / 100) },
                    cta: card.editable ? (todo ? t("home.focus.locale.continue") : t("home.focus.locale.review")) : t("home.focus.locale.browse"),
                    variant: card.editable ? "default" : "outline",
                    go: () => void navigate({ to: "/admin/translations", search: { locale: card.code, filter: card.editable ? "todo" : "all" } }),
                };
            }
            case "notes": {
                const filled = card.total - card.empty;
                return {
                    key: "notes",
                    area: t("home.focus.notes.area"),
                    title: t("home.notes.title", { count: card.empty }),
                    sub: namesSub(t, card.sixStarNames),
                    progress: { value: card.total ? Math.round((filled / card.total) * 100) : 0, label: t("home.focus.notes.progress", { filled: f.number(filled), total: f.number(card.total) }) },
                    cta: t("home.focus.notes.cta"),
                    variant: "default",
                    go: () => void navigate({ to: "/admin/operator-notes", search: { status: "empty", op: card.firstOperatorId } }),
                };
            }
            case "tierList": {
                const { list } = card;
                const hasStats = list.tiers !== undefined && list.placements !== undefined && list.updatedAtMs !== undefined;
                return {
                    key: `tl-${list.slug}`,
                    area: t("home.focus.tierList.area", { level: levelLabel(list.permission) }),
                    title: list.title,
                    sub: hasStats ? t("home.focus.tierList.sub", { tiers: list.tiers ?? 0, placements: list.placements ?? 0, when: f.relative(new Date(list.updatedAtMs ?? 0).toISOString()) }) : null,
                    progress: null,
                    cta: card.canPublish ? t("home.focus.tierList.publish") : t("home.focus.tierList.edit"),
                    variant: "outline",
                    go: () => void navigate({ to: "/admin/tier-lists" }),
                };
            }
        }
    };

    if (cards.length === 0) return <div className="rounded-2xl border border-border border-dashed p-7 text-[14px] text-muted-foreground">{t("home.focus.empty")}</div>;

    return (
        <div className="grid grid-cols-[repeat(auto-fill,minmax(min(280px,100%),1fr))] gap-3">
            {cards.map(toView).map((c) => (
                <Card key={c.key}>
                    <CardHeader>
                        <CardDescription>{c.area}</CardDescription>
                        <CardTitle>{c.title}</CardTitle>
                    </CardHeader>
                    <CardPanel>
                        <div className="flex flex-col gap-3">
                            {c.sub ? <span className="text-pretty text-[13px] text-muted-foreground">{c.sub}</span> : null}
                            {c.progress ? (
                                <div className="flex items-center gap-2.5">
                                    <div className="flex-1">
                                        <Progress value={c.progress.value} aria-label={c.progress.label} />
                                    </div>
                                    <span className="font-semibold text-[12.5px] tabular-nums">{c.progress.label}</span>
                                </div>
                            ) : null}
                            <div className="flex">
                                <Button size="sm" variant={c.variant} onClick={c.go}>
                                    {c.cta} <span aria-hidden>→</span>
                                </Button>
                            </div>
                        </div>
                    </CardPanel>
                </Card>
            ))}
        </div>
    );
}
