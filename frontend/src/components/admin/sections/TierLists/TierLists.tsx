import { useQuery } from "@tanstack/react-query";
import { Navigate, useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import { useAdminAccess } from "#/components/admin/shell/access";
import { PageHead } from "#/components/admin/shell/PageHead";
import type { AdminTierListsTab, IAdminTierListsSearch } from "#/components/admin/shell/search";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Tabs, TabsList, TabsTab } from "#/components/ui/tabs";
import { useAuth } from "#/hooks/use-auth";
import { allTierListGrantsQueryOptions } from "#/lib/api/admin";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { AccessTab } from "./AccessTab";
import { NewListDialog } from "./dialogs";
import { ListsTab } from "./ListsTab";
import type { messages } from "./TierLists.messages";

export interface ITierListsProps {
    search: IAdminTierListsSearch;
}

/**
 * Admin > Tier lists (spec 2.3). Staff manage every list and the per-list
 * grants; an editor sees the lists granted to them at their level. Tab,
 * filter and query live in the URL.
 */
export default function TierLists({ search }: ITierListsProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("admin");
    const f = useFormatters();
    const { isAuthenticated } = useAuth();
    const { staff, can, tierListGrantsLoading } = useAdminAccess();
    const navigate = useNavigate({ from: "/admin/tier-lists" });
    const [showNew, setShowNew] = useState(false);

    // The Access tab and its count are staff only: the endpoint 403s for anyone else.
    const grantsQuery = useQuery({ ...allTierListGrantsQueryOptions({}, isAuthenticated && staff), enabled: isAuthenticated && staff });
    const tab: AdminTierListsTab = staff ? (search.tab ?? "lists") : "lists";

    const setTab = (next: AdminTierListsTab): void => {
        void navigate({ search: (prev) => ({ ...prev, tab: next === "lists" ? undefined : next }), replace: true });
    };

    // Mirrors Translations: once the grants are in, a user holding none goes home.
    if (!tierListGrantsLoading && !can.tierlists) return <Navigate to="/admin" replace />;

    return (
        <>
            <PageHead
                kicker={t("tierLists.head.kicker")}
                title={staff ? t("tierLists.head.titleStaff") : t("tierLists.head.titleOwn")}
                sub={staff ? t("tierLists.head.subStaff") : t("tierLists.head.subOwn")}
                action={
                    staff ? (
                        <Button size="sm" onClick={() => setShowNew(true)}>
                            {t("tierLists.head.newList")}
                        </Button>
                    ) : null
                }
            />
            <div className="flex flex-col gap-3.5">
                {staff ? (
                    <Tabs value={tab} onValueChange={(v) => setTab(v as AdminTierListsTab)}>
                        <TabsList variant="underline">
                            <TabsTab value="lists">{t("tierLists.tabs.lists")}</TabsTab>
                            <TabsTab value="access">
                                {t("tierLists.tabs.access")}
                                {grantsQuery.data ? (
                                    <Badge variant="secondary" size="sm">
                                        {f.number(grantsQuery.data.length)}
                                    </Badge>
                                ) : null}
                            </TabsTab>
                        </TabsList>
                    </Tabs>
                ) : null}
                {tab === "access" ? <AccessTab query={grantsQuery} /> : <ListsTab search={search} staff={staff} />}
            </div>
            {staff ? <NewListDialog open={showNew} onOpenChange={setShowNew} /> : null}
        </>
    );
}
