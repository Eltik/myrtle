import { RefreshCw } from "lucide-react";
import { Button } from "~/components/ui/shadcn/button";
import type { AuthUser } from "~/hooks/use-auth";
import { cn } from "~/lib/utils";
import type { AdminRole } from "~/types/frontend/impl/admin";

const ROLE_BADGE_STYLES: Record<AdminRole, string> = {
    tier_list_editor: "bg-blue-500/10 text-blue-500",
    tier_list_admin: "bg-purple-500/10 text-purple-500",
    super_admin: "bg-amber-500/10 text-amber-500",
};

const ROLE_DISPLAY_NAMES: Record<AdminRole, string> = {
    tier_list_editor: "Tier List Editor",
    tier_list_admin: "Tier List Admin",
    super_admin: "Super Admin",
};

export function Header({ user, role, statsLoading, onRefresh }: { user: AuthUser; role: AdminRole; statsLoading: boolean; onRefresh: () => void }) {
    return (
        <div className="flex items-center justify-between">
            <div>
                <h2 className="scroll-m-20 border-b pb-2 font-semibold text-3xl tracking-tight first:mt-0">Welcome back, {user.status.nickName}!</h2>
                <p className="text-muted-foreground text-sm leading-7">Lets get started.</p>
                <span className={cn("rounded px-2 py-0.5 font-medium font-mono text-xs", ROLE_BADGE_STYLES[role])}>{ROLE_DISPLAY_NAMES[role]}</span>
            </div>
            <Button disabled={statsLoading} onClick={onRefresh} size="sm" variant="outline">
                <RefreshCw className={`mr-2 h-4 w-4 ${statsLoading ? "animate-spin" : ""}`} />
                Refresh
            </Button>
        </div>
    );
}
