import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import { PanelMessage } from "#/components/admin/Primitives";
import type { IAdminAccess } from "#/components/admin/shell/access";
import type { InboxItem, IOrphanLocaleGrant } from "#/components/admin/shell/inbox";
import { invalidateLocaleGrants } from "#/components/admin/shell/invalidate";
import { useAdminLocales } from "#/components/admin/shell/locales";
import { useInboxQueue } from "#/components/admin/shell/useInboxQueue";
import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle } from "#/components/ui/alert-dialog";
import { Button } from "#/components/ui/button";
import { Card, CardDescription, CardHeader, CardPanel, CardTitle } from "#/components/ui/card";
import { toastManager } from "#/components/ui/toast";
import { revokeTranslationPermissionFn, type TierListPermissionLevel } from "#/lib/api/admin";
import { useT } from "#/lib/i18n";
import { type HomeT, sixStarsSub } from "./useHomeLabels";

interface IRow {
    key: string;
    area: string;
    dot: string;
    title: string;
    sub: string;
    cta: string;
    /** False for a row whose button acts in place instead of opening a page. */
    navigates: boolean;
    go: () => void;
}

/** The staff "Needs attention" card (design `queue`): one row per thing waiting on an admin. */
export function InboxCard({ access }: { access: IAdminAccess }): React.ReactElement {
    const t: HomeT = useT("admin");
    const navigate = useNavigate();
    const items = useInboxQueue(access);
    const locales = useAdminLocales(access.isSuper);
    // The orphan grants the clean-up dialog is confirming; null = closed.
    const [cleanup, setCleanup] = useState<IOrphanLocaleGrant[] | null>(null);

    const toRow = (item: InboxItem, i: number): IRow => {
        switch (item.kind) {
            case "translationsStale": {
                const language = locales.name(item.locale);
                return {
                    key: `stale-${item.locale}`,
                    area: t("home.queue.area.translations"),
                    dot: "var(--warning)",
                    title: t("home.queue.stale.title", { count: item.count, language }),
                    sub: t("home.queue.stale.sub"),
                    cta: t("home.queue.stale.cta"),
                    navigates: true,
                    go: () => void navigate({ to: "/admin/translations", search: { locale: item.locale, filter: "stale" } }),
                };
            }
            case "translationsMissing": {
                const language = locales.name(item.locale);
                return {
                    key: `missing-${item.locale}`,
                    area: t("home.queue.area.translations"),
                    dot: "var(--chart-3)",
                    title: t("home.queue.missing.title", { count: item.count, language }),
                    sub: t("home.queue.missing.sub", { language }),
                    cta: t("home.queue.missing.cta"),
                    navigates: true,
                    go: () => void navigate({ to: "/admin/translations", search: { locale: item.locale, filter: "untranslated" } }),
                };
            }
            case "notesEmpty":
                return {
                    key: "notes",
                    area: t("home.queue.area.notes"),
                    dot: "var(--chart-2)",
                    title: t("home.notes.title", { count: item.count }),
                    sub: sixStarsSub(t, item.sixStarNames),
                    cta: t("home.queue.notes.cta"),
                    navigates: true,
                    go: () => void navigate({ to: "/admin/operator-notes", search: { status: "empty", op: item.firstOperatorId } }),
                };
            case "orphanLocaleGrants":
                return {
                    key: "orphan-locale-grants",
                    area: t("home.queue.area.people"),
                    dot: "var(--primary)",
                    title: t("home.queue.orphans.title", { count: item.grants.length }),
                    sub: t("home.queue.orphans.sub"),
                    cta: t("home.queue.orphans.cta"),
                    navigates: false,
                    go: () => setCleanup(item.grants),
                };
            case "systemDegraded":
                return {
                    key: `system-${i}`,
                    area: t("home.queue.area.system"),
                    dot: "var(--destructive)",
                    title: t("home.queue.system.title"),
                    sub: t("home.queue.system.sub"),
                    cta: t("home.queue.system.cta"),
                    navigates: true,
                    go: () => void navigate({ to: "/admin/system", search: { tab: "health" } }),
                };
        }
    };

    const rows = items?.map(toRow);

    return (
        <Card>
            <CardHeader>
                <CardTitle>{t("home.queue.title")}</CardTitle>
                <CardDescription>{rows === undefined ? " " : rows.length ? t("home.queue.desc", { count: rows.length }) : t("home.queue.descEmpty")}</CardDescription>
            </CardHeader>
            <CardPanel>
                <div className="flex flex-col">
                    {rows === undefined ? (
                        <PanelMessage className="py-7">{t("home.queue.loading")}</PanelMessage>
                    ) : rows.length === 0 ? (
                        <PanelMessage className="py-7">{t("home.queue.empty")}</PanelMessage>
                    ) : (
                        rows.map((row) => (
                            <div key={row.key} className="relative -mx-2.5 grid grid-cols-[minmax(0,1fr)_auto] items-center gap-x-3.5 gap-y-1.5 rounded-lg border-border border-t px-2.5 py-[13px] hover:bg-accent md:grid-cols-[118px_minmax(0,1fr)_auto]">
                                {/* Covers the row so all of it is clickable; keyboard users get the button. */}
                                <button type="button" tabIndex={-1} aria-hidden className="absolute inset-0 cursor-pointer rounded-lg" onClick={row.go} />
                                <span className="col-span-full flex items-center gap-2 font-medium text-[12.5px] text-muted-foreground md:col-auto">
                                    <span className="size-[7px] shrink-0 rounded-full" style={{ background: row.dot }} />
                                    {row.area}
                                </span>
                                <div className="flex min-w-0 flex-col gap-0.5">
                                    <span className="font-medium text-[14px]">{row.title}</span>
                                    <span className="text-[13px] text-muted-foreground">{row.sub}</span>
                                </div>
                                <Button size="sm" variant="outline" className="z-10" onClick={row.go}>
                                    {row.cta}
                                    {row.navigates ? <span aria-hidden> →</span> : null}
                                </Button>
                            </div>
                        ))
                    )}
                </div>
            </CardPanel>
            <OrphanCleanupDialog grants={cleanup} onClose={() => setCleanup(null)} />
        </Card>
    );
}

/**
 * Confirms, then revokes every language grant whose account is gone. Only a
 * super-admin sees the row that opens it, and only they may revoke.
 */
function OrphanCleanupDialog({ grants, onClose }: { grants: IOrphanLocaleGrant[] | null; onClose: () => void }): React.ReactElement {
    const t: HomeT = useT("admin");
    const queryClient = useQueryClient();
    // Keep the count on screen while the dialog fades out.
    const [last, setLast] = useState<IOrphanLocaleGrant[] | null>(null);
    if (grants !== null && grants !== last) setLast(grants);
    const count = (grants ?? last)?.length ?? 0;

    const revokeAll = useMutation({
        mutationFn: async (targets: IOrphanLocaleGrant[]) => {
            const results = await Promise.allSettled(targets.map((g) => revokeTranslationPermissionFn({ data: { locale: g.locale, userId: g.user_id, permission: g.permission as TierListPermissionLevel } })));
            return { removed: results.filter((r) => r.status === "fulfilled").length, failed: results.filter((r) => r.status === "rejected").length };
        },
        onSettled: () => invalidateLocaleGrants(queryClient),
        onSuccess: ({ removed, failed }) => {
            onClose();
            if (failed === 0) toastManager.add({ id: `orphan-cleanup-${Date.now()}`, title: t("home.cleanup.toast.done", { count: removed }), type: "success" });
            else toastManager.add({ id: `orphan-cleanup-err-${Date.now()}`, title: t("home.cleanup.toast.partial", { count: failed }), description: t("home.cleanup.toast.partial.desc"), type: "error" });
        },
    });

    return (
        <AlertDialog open={grants !== null} onOpenChange={(open) => (open || revokeAll.isPending ? undefined : onClose())}>
            <AlertDialogPopup>
                <AlertDialogHeader>
                    <AlertDialogTitle>{t("home.cleanup.title", { count })}</AlertDialogTitle>
                    <AlertDialogDescription>{t("home.cleanup.desc", { count })}</AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                    <AlertDialogClose render={<Button type="button" variant="outline" disabled={revokeAll.isPending} />}>{t("home.cleanup.cancel")}</AlertDialogClose>
                    <Button type="button" variant="destructive" loading={revokeAll.isPending} onClick={() => grants && revokeAll.mutate(grants)}>
                        {t("home.cleanup.confirm", { count })}
                    </Button>
                </AlertDialogFooter>
            </AlertDialogPopup>
        </AlertDialog>
    );
}
