import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { RefreshCwIcon } from "lucide-react";
import { useState } from "react";
import { useAdminAccess } from "#/components/admin/shell/access";
import { PageHead } from "#/components/admin/shell/PageHead";
import type { AdminSystemTab, IAdminSystemSearch } from "#/components/admin/shell/search";
import { Button } from "#/components/ui/button";
import { useErrorMessage } from "#/components/ui/error-message";
import { Tabs, TabsList, TabsTab } from "#/components/ui/tabs";
import { toastManager } from "#/components/ui/toast";
import { useAuth } from "#/hooks/use-auth";
import { healthQueryOptions } from "#/lib/api/admin";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { AuditTab } from "./AuditTab";
import { HealthTab, useHealthCopy } from "./HealthTab";
import { summarizeHealth } from "./health";
import { LanguagesTab } from "./LanguagesTab";
import type { messages } from "./System.messages";

export interface ISystemProps {
    search: IAdminSystemSearch;
}

/**
 * System (spec 2.6): service health, the merged edit history and the site
 * languages. Staff only; the route already redirects everyone else, and every
 * query here is gated on `staff` as well so no role fires a request that 403s.
 */
export default function System({ search }: ISystemProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("admin");
    const navigate = useNavigate();
    const describeError = useErrorMessage();
    const queryClient = useQueryClient();
    const healthCopy = useHealthCopy();
    const { isAuthenticated } = useAuth();
    const access = useAdminAccess();
    const enabled = isAuthenticated && access.staff;
    const tab: AdminSystemTab = search.tab ?? "health";

    const healthQuery = useQuery({ ...healthQueryOptions(), enabled });
    const [checking, setChecking] = useState(false);

    const checkNow = async (): Promise<void> => {
        setChecking(true);
        try {
            void queryClient.invalidateQueries({ queryKey: ["admin", "stats"] });
            const result = await healthQuery.refetch();
            const id = `system-health-${Date.now()}`;
            if (result.isError || !result.data) {
                toastManager.add({ id, title: t("system.toast.failed.title"), description: describeError(result.error), type: "error" });
                return;
            }
            const summary = summarizeHealth(result.data, false);
            if (summary === "healthy") toastManager.add({ id, title: t("system.toast.healthy.title"), description: t("system.toast.healthy.desc"), type: "success" });
            else toastManager.add({ id, title: t("system.toast.unhealthy.title"), description: `${healthCopy.status(summary)}. ${healthCopy.detail(summary)}`, type: "error" });
        } finally {
            setChecking(false);
        }
    };

    const selectTab = (next: AdminSystemTab): void => {
        void navigate({ to: "/admin/system", search: next === "health" ? {} : { tab: next } });
    };

    return (
        <>
            <PageHead
                kicker={t("system.head.kicker")}
                title={t("system.head.title")}
                sub={t("system.head.sub")}
                action={
                    <Button variant="outline" size="sm" loading={checking} disabled={!enabled || checking} onClick={() => void checkNow()}>
                        <RefreshCwIcon />
                        {t("system.head.checkNow")}
                    </Button>
                }
            />
            <div className="flex flex-col gap-4">
                <Tabs value={tab} onValueChange={(value) => selectTab(value as AdminSystemTab)}>
                    <TabsList variant="underline">
                        <TabsTab value="health">{t("system.tabs.health")}</TabsTab>
                        <TabsTab value="audit">{t("system.tabs.audit")}</TabsTab>
                        <TabsTab value="languages">{t("system.tabs.languages")}</TabsTab>
                    </TabsList>
                </Tabs>
                {tab === "health" ? <HealthTab enabled={enabled} /> : null}
                {tab === "audit" ? <AuditTab enabled={enabled} /> : null}
                {tab === "languages" ? <LanguagesTab enabled={enabled} canEdit={access.isSuper} /> : null}
            </div>
        </>
    );
}
