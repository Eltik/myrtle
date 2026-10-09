import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import { roleVariant, useLevelLabel } from "#/components/admin/sections/Users/labels";
import type { IAdminAccess } from "#/components/admin/shell/access";
import { useAdminLocales } from "#/components/admin/shell/locales";
import { levelVariant } from "#/components/admin/shell/model";
import { useRoleLabel } from "#/components/admin/shell/nav";
import { Badge, type IBadgeProps } from "#/components/ui/badge";
import { Card, CardDescription, CardHeader, CardPanel, CardTitle } from "#/components/ui/card";
import { useAuth } from "#/hooks/use-auth";
import { globalAuditLogQueryOptions, translationAuditLogQueryOptions } from "#/lib/api/admin";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { grantedTierListsQueryOptions } from "#/lib/api/tier-lists";
import { useFormatters, useGamedataServer, useT } from "#/lib/i18n";
import { mergeRecentChanges } from "./queue";
import { type HomeT, useNoteFieldLabel } from "./useHomeLabels";

const RECENT_LIMIT = 5;
/** A translator's own rows are filtered client-side from the shared feed, so read further back. */
const OWN_TRANSLATION_WINDOW = 100;

interface IAccessRow {
    key: string;
    label: string;
    level: string;
    variant: NonNullable<IBadgeProps["variant"]>;
}

/** "What you can do": the signed-in person's own role and grants. */
export function AccessCard({ access }: { access: IAdminAccess }): React.ReactElement {
    const t: HomeT = useT("admin");
    const roleLabel = useRoleLabel();
    const levelLabel = useLevelLabel();
    const { isAuthenticated } = useAuth();
    const role = access.role ?? "user";
    const grantedQuery = useQuery({ ...grantedTierListsQueryOptions(isAuthenticated), enabled: isAuthenticated && !access.staff });
    const locales = useAdminLocales(role === "translator");

    const note = role === "super_admin" ? t("home.access.note.superAdmin") : role === "tier_list_admin" ? t("home.access.note.tierListAdmin") : role === "tier_list_editor" ? t("home.access.note.tierListEditor") : role === "translator" ? t("home.access.note.translator") : null;

    const rows: IAccessRow[] = access.isSuper
        ? [{ key: "everything", label: t("home.access.everything"), level: roleLabel(role), variant: "default" }]
        : [
              { key: "role", label: t("home.access.siteRole"), level: roleLabel(role), variant: roleVariant(role) },
              ...(role === "tier_list_admin" ? [{ key: "lists", label: t("home.access.everyList"), level: levelLabel("admin"), variant: levelVariant("admin") }] : []),
              ...(role === "tier_list_editor" ? [{ key: "notes", label: t("home.access.notes"), level: levelLabel("edit"), variant: levelVariant("edit") }] : []),
              ...(access.staff ? [] : (grantedQuery.data ?? []).map((g) => ({ key: `tl-${g.slug}`, label: g.title, level: levelLabel(g.permission), variant: levelVariant(g.permission) }))),
              ...access.myLoc.map((g) => ({ key: `loc-${g.code}`, label: locales.name(g.code), level: levelLabel(g.level), variant: levelVariant(g.level) })),
          ];

    return (
        <Card>
            <CardHeader>
                <CardTitle>{t("home.access.title")}</CardTitle>
                {note ? <CardDescription>{note}</CardDescription> : null}
            </CardHeader>
            <CardPanel>
                <div className="flex flex-col">
                    {rows.map((r) => (
                        <div key={r.key} className="flex items-center gap-2.5 border-border border-t py-[9px]">
                            <span className="flex-1 text-[13.5px]">{r.label}</span>
                            <Badge variant={r.variant}>{r.level}</Badge>
                        </div>
                    ))}
                </div>
            </CardPanel>
        </Card>
    );
}

/** The latest saved edits: everyone's for staff, your own otherwise. */
export function RecentCard({ access }: { access: IAdminAccess }): React.ReactElement {
    const t: HomeT = useT("admin");
    const f = useFormatters();
    const fieldLabel = useNoteFieldLabel();
    const { user: me, isAuthenticated } = useAuth();
    const isEditor = access.role === "tier_list_editor";
    const isTranslator = access.role === "translator";
    // The notes audit is admin-role only (an editor reads their own rows); the
    // translation audit is any panel role, but only roles that translate need it.
    const wantNotes = isAuthenticated && (access.staff || isEditor);
    const wantTranslations = isAuthenticated && (access.staff || isTranslator);

    const notesQuery = useQuery({ ...globalAuditLogQueryOptions(access.staff ? { limit: RECENT_LIMIT } : { limit: RECENT_LIMIT, actor: "me" }, me?.id), enabled: wantNotes });
    const i18nQuery = useQuery({ ...translationAuditLogQueryOptions({ limit: access.staff ? RECENT_LIMIT : OWN_TRANSLATION_WINDOW }, isAuthenticated), enabled: wantTranslations });
    const opsQuery = useQuery({ ...operatorsIndexQueryOptions(useGamedataServer()), enabled: wantNotes });
    const locales = useAdminLocales(wantTranslations);

    const opName = useMemo(() => {
        const names = new Map((opsQuery.data ?? []).map((o) => [o.id, o.name]));
        return (id: string) => names.get(id) ?? id;
    }, [opsQuery.data]);

    const recent = mergeRecentChanges(notesQuery.data?.entries ?? [], i18nQuery.data?.entries ?? [], { onlyActorId: access.staff ? undefined : (me?.id ?? ""), limit: RECENT_LIMIT });

    const text = (r: (typeof recent)[number]) => {
        if (r.kind === "note") {
            const values = { actor: r.actorName ?? t("home.recent.deletedAccount"), operator: opName(r.operatorId), field: fieldLabel(r.field) };
            return access.staff ? t("home.recent.note", values) : t("home.recent.noteOwn", values);
        }
        const values = { actor: r.actorName ?? t("home.recent.deletedAccount"), language: locales.name(r.locale), key: r.key };
        return access.staff ? t("home.recent.translation", values) : t("home.recent.translationOwn", values);
    };

    return (
        <Card>
            <CardHeader>
                <CardTitle>{access.staff ? t("home.recent.titleStaff") : t("home.recent.titleOwn")}</CardTitle>
            </CardHeader>
            <CardPanel>
                <div className="flex flex-col">
                    {recent.map((r) => {
                        const line = text(r);
                        return (
                            <div key={r.id} className="flex items-center gap-2.5 border-border border-t py-[9px]">
                                <span className="min-w-0 flex-1 truncate text-[13.5px]" title={line}>
                                    {line}
                                </span>
                                <span className="whitespace-nowrap text-[12.5px] text-muted-foreground">{f.relative(r.at)}</span>
                            </div>
                        );
                    })}
                    {recent.length === 0 ? <span className="text-[13px] text-muted-foreground">{t("home.recent.empty")}</span> : null}
                </div>
            </CardPanel>
        </Card>
    );
}
