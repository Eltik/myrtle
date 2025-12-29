import { RefreshCw } from "lucide-react";
import { Button } from "~/components/ui/shadcn/button";
import type { AuthUser } from "~/hooks/use-auth";
import type { AdminRole } from "~/types/frontend/admin";

export function Header({ user, role, statsLoading, onRefresh }: { user: AuthUser; role: AdminRole; statsLoading: boolean; onRefresh: () => void }) {
    return (
        <div className="flex items-center justify-between">
            <div>
                <h2 className="scroll-m-20 border-b pb-2 font-semibold text-3xl tracking-tight first:mt-0">Welcome back, {user.status.nickName}!</h2>
                <p className="text-muted-foreground text-sm leading-7">Lets get started.</p>
                <span className="rounded bg-primary/10 px-2 py-0.5 font-medium font-mono text-primary text-xs">{role}</span>
            </div>
            <Button disabled={statsLoading} onClick={onRefresh} size="sm" variant="outline">
                <RefreshCw className={`mr-2 h-4 w-4 ${statsLoading ? "animate-spin" : ""}`} />
                Refresh
            </Button>
        </div>
    );
}
