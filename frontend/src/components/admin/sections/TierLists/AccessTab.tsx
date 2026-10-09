import { type UseQueryResult, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { ChevronDownIcon } from "lucide-react";
import { useMemo, useState } from "react";
import { PanelMessage, stackedCardAction, stackedCardHeader, stickyActionsCell } from "#/components/admin/Primitives";
import { invalidateTierListGrants } from "#/components/admin/shell/invalidate";
import { GRANT_LEVELS, levelVariant } from "#/components/admin/shell/model";
import { toastError, toastSuccess } from "#/components/admin/shell/toast";
import { Badge, badgeVariants } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card, CardAction, CardDescription, CardHeader, CardTitle } from "#/components/ui/card";
import { useErrorMessage } from "#/components/ui/error-message";
import { DropdownMenu, DropdownMenuContent, DropdownMenuRadioGroup, DropdownMenuRadioItem, DropdownMenuTrigger } from "#/components/ui/menu";
import { Skeleton } from "#/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "#/components/ui/table";
import { grantTierListPermissionFn, type ITierListGrant, revokeTierListPermissionFn, type TierListPermissionLevel } from "#/lib/api/admin";
import { browseTierListsQueryOptions } from "#/lib/api/tier-lists";
import { useFormatters, useGamedataServer, useT } from "#/lib/i18n";
import { cn } from "#/lib/utils";
import { GrantDialog, type IGrantListOption } from "./GrantDialog";
import { type TierListsT, useLevelLabel } from "./labels";
import { levelChangePlan } from "./model";

/** Staff only (the parent renders it for staff alone, and its query is gated the same way). */
export function AccessTab({ query }: { query: UseQueryResult<ITierListGrant[]> }): React.ReactElement {
    const t: TierListsT = useT("admin");
    const f = useFormatters();
    const server = useGamedataServer();
    const browseQuery = useQuery(browseTierListsQueryOptions(server));
    const [granting, setGranting] = useState(false);

    // Every listed list of either type, plus any list that only appears through a grant (unlisted).
    const listOptions = useMemo((): IGrantListOption[] => {
        const bySlug = new Map<string, IGrantListOption>();
        for (const l of browseQuery.data ?? []) bySlug.set(l.slug, { slug: l.slug, title: l.title });
        for (const g of query.data ?? []) if (!bySlug.has(g.slug)) bySlug.set(g.slug, { slug: g.slug, title: g.title });
        return [...bySlug.values()].sort((a, b) => f.collator.compare(a.title, b.title));
    }, [browseQuery.data, query.data, f.collator]);

    const grants = query.data ?? [];

    return (
        <Card>
            <CardHeader className={stackedCardHeader}>
                <CardTitle>{t("tierLists.access.title")}</CardTitle>
                <CardDescription>{t("tierLists.access.desc")}</CardDescription>
                <CardAction className={stackedCardAction}>
                    <Button size="sm" variant="outline" onClick={() => setGranting(true)}>
                        {t("tierLists.access.grant")}
                    </Button>
                </CardAction>
            </CardHeader>
            {query.isError ? (
                <PanelMessage className="p-8">{t("tierLists.access.loadError")}</PanelMessage>
            ) : (
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead className="pl-6">{t("tierLists.access.th.player")}</TableHead>
                            <TableHead>{t("tierLists.access.th.list")}</TableHead>
                            <TableHead>{t("tierLists.access.th.level")}</TableHead>
                            <TableHead className="max-md:hidden">{t("tierLists.access.th.granted")}</TableHead>
                            <TableHead className={stickyActionsCell}>
                                <span className="sr-only">{t("tierLists.access.th.actions")}</span>
                            </TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {query.isPending
                            ? [0, 1, 2].map((i) => (
                                  <TableRow key={i}>
                                      <TableCell colSpan={5}>
                                          <Skeleton className="h-8" />
                                      </TableCell>
                                  </TableRow>
                              ))
                            : grants.map((g) => <GrantRow key={`${g.tierListId}-${g.userId}-${g.permission}`} grant={g} />)}
                    </TableBody>
                </Table>
            )}
            {query.isSuccess && grants.length === 0 ? <PanelMessage className="p-8">{t("tierLists.access.empty")}</PanelMessage> : null}
            <GrantDialog open={granting} onOpenChange={setGranting} lists={listOptions} />
        </Card>
    );
}

function GrantRow({ grant }: { grant: ITierListGrant }): React.ReactElement {
    const t: TierListsT = useT("admin");
    const levelLabel = useLevelLabel();
    const f = useFormatters();
    const describeError = useErrorMessage();
    const queryClient = useQueryClient();
    const name = grant.userNickname ?? t("tierLists.access.unnamed");
    const target = { slug: grant.slug, userId: grant.userId };

    const revoke = useMutation({
        mutationFn: () => revokeTierListPermissionFn({ data: { ...target, permission: grant.permission } }),
        onSuccess: () => {
            invalidateTierListGrants(queryClient);
            toastSuccess("tier-lists-revoked", t("tierLists.access.toast.revoked"), t("tierLists.access.toast.revokedDesc", { name, title: grant.title }));
        },
        onError: (err: unknown) => toastError("tier-lists-revoke-failed", t("tierLists.access.toast.revokeFailed"), describeError(err)),
    });

    // A grant row is keyed by its level, so a change grants the new row first, then drops the old one:
    // a failure in between leaves the player with both levels rather than none.
    const changeLevel = useMutation({
        mutationFn: async (plan: { grant: TierListPermissionLevel; revoke: TierListPermissionLevel }) => {
            await grantTierListPermissionFn({ data: { ...target, permission: plan.grant } });
            await revokeTierListPermissionFn({ data: { ...target, permission: plan.revoke } });
            return plan.grant;
        },
        onSuccess: (level) => {
            toastSuccess("tier-lists-level", t("tierLists.access.toast.levelChanged"), t("tierLists.access.toast.levelDesc", { name, level: levelLabel(level), title: grant.title }));
        },
        onError: (err: unknown) => toastError("tier-lists-level-failed", t("tierLists.access.toast.levelFailed"), describeError(err)),
        onSettled: () => invalidateTierListGrants(queryClient),
    });

    const busy = revoke.isPending || changeLevel.isPending;

    return (
        <TableRow>
            <TableCell className="pl-6">
                <div className="flex flex-col gap-0.75">
                    <span className="font-medium">{name}</span>
                    <span className="whitespace-nowrap font-mono text-[12px] text-muted-foreground leading-[1.2]">{t("tierLists.access.uid", { uid: grant.userUid })}</span>
                </div>
            </TableCell>
            <TableCell>
                <span className="flex items-center gap-1.5">
                    {grant.title}
                    {grant.listType === "community" ? (
                        <Badge variant="outline" size="sm">
                            {t("tierLists.lists.community")}
                        </Badge>
                    ) : null}
                </span>
            </TableCell>
            <TableCell>
                <DropdownMenu>
                    <DropdownMenuTrigger disabled={busy} aria-label={t("tierLists.access.changeLevel", { name, title: grant.title })} className={cn(badgeVariants({ variant: levelVariant(grant.permission, "default") }), "pointer-coarse:min-h-9 cursor-pointer gap-0.5 pointer-coarse:px-2.5 pe-0.5")}>
                        {levelLabel(grant.permission)}
                        <ChevronDownIcon className="size-3!" />
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="start">
                        <DropdownMenuRadioGroup
                            value={grant.permission}
                            onValueChange={(next: TierListPermissionLevel) => {
                                const plan = levelChangePlan(grant.permission, next);
                                if (plan) changeLevel.mutate(plan);
                            }}
                        >
                            {GRANT_LEVELS.map((level) => (
                                <DropdownMenuRadioItem key={level} value={level}>
                                    {levelLabel(level)}
                                </DropdownMenuRadioItem>
                            ))}
                        </DropdownMenuRadioGroup>
                    </DropdownMenuContent>
                </DropdownMenu>
            </TableCell>
            <TableCell className="max-md:hidden">
                <span className="text-muted-foreground">{t("tierLists.access.grantedBy", { name: grant.grantedByNickname ?? t("tierLists.access.unknownGranter"), when: f.relative(grant.grantedAt) })}</span>
            </TableCell>
            <TableCell className={stickyActionsCell}>
                <div className="flex justify-end gap-1.5 pr-2.5">
                    <Button size="xs" variant="ghost" render={<Link to="/admin/users" search={{ user: grant.userId }} />}>
                        {t("tierLists.access.profile")}
                    </Button>
                    <Button size="xs" variant="ghost" loading={revoke.isPending} disabled={busy} onClick={() => revoke.mutate()}>
                        {t("tierLists.access.revoke")}
                    </Button>
                </div>
            </TableCell>
        </TableRow>
    );
}
