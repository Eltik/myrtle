import { useQuery } from "@tanstack/react-query";
import { Navigate, useNavigate } from "@tanstack/react-router";
import { useCallback, useMemo } from "react";
import { useAdminAccess } from "#/components/admin/shell/access";
import { atLeast } from "#/components/admin/shell/model";
import { PageHead } from "#/components/admin/shell/PageHead";
import type { IAdminTranslationsSearch } from "#/components/admin/shell/search";
import { Card } from "#/components/ui/card";
import { Skeleton } from "#/components/ui/skeleton";
import { useAuth } from "#/hooks/use-auth";
import { useIsMac } from "#/hooks/use-is-mac";
import { localesQueryOptions, translationNamespacesQueryOptions, translationProgressQueryOptions } from "#/lib/api/admin";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { LOCALE_PICKER, LOCALE_PICKER_ITEM, LocaleCard, useSelectedCardInView } from "./LocalePicker";
import { localeStats, resolveFilter, resolveLocale } from "./model";
import { StringsPane } from "./StringsPane";
import type { messages } from "./Translations.messages";

type T = TypedT<typeof messages>;

export interface ITranslationsProps {
    search: IAdminTranslationsSearch;
}

export default function Translations({ search }: ITranslationsProps): React.ReactElement {
    const t: T = useT("admin");
    const isMac = useIsMac();
    const access = useAdminAccess();
    const head = <PageHead kicker={t("translations.head.kicker")} title={t("translations.head.title")} sub={t("translations.head.sub", { shortcut: isMac ? "⌘↵" : "Ctrl↵" })} />;

    // The route admits every translator; one holding no grant has nothing here (design: fall back to Home).
    if (!access.localeGrantsLoading && !access.can.translations) return <Navigate to="/admin" replace />;

    return (
        <>
            {head}
            <Workspace search={search} grants={access.myLoc} grantsLoading={access.localeGrantsLoading} enabled={access.can.translations} />
        </>
    );
}

interface IWorkspaceProps {
    search: IAdminTranslationsSearch;
    grants: readonly { code: string; level: string }[];
    grantsLoading: boolean;
    enabled: boolean;
}

function Workspace({ search, grants, grantsLoading, enabled: canOpen }: IWorkspaceProps): React.ReactElement {
    const t: T = useT("admin");
    const { isAuthenticated } = useAuth();
    const navigate = useNavigate({ from: "/admin/translations" });
    const enabled = isAuthenticated && canOpen;

    const localesQuery = useQuery({ ...localesQueryOptions(isAuthenticated), enabled });
    const progressQuery = useQuery({ ...translationProgressQueryOptions(isAuthenticated), enabled });
    const namespacesQuery = useQuery({ ...translationNamespacesQueryOptions(isAuthenticated), enabled });

    // Filters and Save & next replace the entry; opening or leaving a string pushes one, so on a
    // phone the browser's Back returns from the editor to the list instead of leaving the section.
    const setSearch = useCallback(
        (patch: Partial<IAdminTranslationsSearch>, replace = true) => {
            void navigate({ search: (prev) => ({ ...prev, ...patch }), replace, resetScroll: false });
        },
        [navigate],
    );

    const levels = useMemo(() => new Map(grants.map((g) => [g.code, g.level])), [grants]);
    // English is the source: it has nothing to translate, so it gets no card.
    const visible = useMemo(() => (localesQuery.data ?? []).filter((l) => !l.is_source && levels.has(l.code)).sort((a, b) => a.sort_order - b.sort_order || a.code.localeCompare(b.code)), [localesQuery.data, levels]);
    const localeCode = resolveLocale(
        visible.map((l) => l.code),
        search.locale,
    );
    const locale = visible.find((l) => l.code === localeCode);
    const canEdit = localeCode !== undefined && atLeast(levels.get(localeCode) ?? "", "edit");
    const filter = resolveFilter(search.filter, canEdit);
    const progressRow = progressQuery.data?.find((p) => p.locale === localeCode);
    const stats = localeStats(progressRow);

    const pickerRef = useSelectedCardInView(locale !== undefined);

    if (grantsLoading || localesQuery.isPending) {
        return (
            <div className={LOCALE_PICKER}>
                <Skeleton className={cn("h-[110px] rounded-2xl", LOCALE_PICKER_ITEM)} />
                <Skeleton className={cn("h-[110px] rounded-2xl", LOCALE_PICKER_ITEM)} />
                <Skeleton className={cn("h-[110px] rounded-2xl", LOCALE_PICKER_ITEM)} />
            </div>
        );
    }

    if (!locale) {
        return (
            <Card>
                <p className="px-6 py-10 text-center text-[13.5px] text-muted-foreground">{t("translations.locale.none")}</p>
            </Card>
        );
    }

    return (
        <div className="flex flex-col gap-4">
            <div ref={pickerRef} className={LOCALE_PICKER}>
                {visible.map((l) => (
                    <LocaleCard key={l.code} locale={l} progress={progressQuery.data?.find((p) => p.locale === l.code)} progressPending={progressQuery.isPending} viewOnly={!atLeast(levels.get(l.code) ?? "", "edit")} selected={l.code === locale.code} onSelect={() => setSearch({ locale: l.code, key: undefined })} />
                ))}
            </div>
            {/* `key` remounts the list on a locale switch so its search box and scroll start fresh. */}
            <StringsPane key={locale.code} search={search} locale={locale} canEdit={canEdit} filter={filter} stats={stats} statsReady={progressRow !== undefined} namespaces={namespacesQuery.data} enabled={enabled} setSearch={setSearch} />
        </div>
    );
}
