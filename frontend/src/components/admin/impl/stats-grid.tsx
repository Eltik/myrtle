import { Activity, History, LayoutList, MoveDownRight, MoveUpRight, Users } from "lucide-react";
import { Separator } from "~/components/ui/shadcn/separator";
import type { AdminRole } from "~/lib/permissions";
import { canManageUsers } from "~/lib/permissions";
import { cn } from "~/lib/utils";
import type { AdminStats } from "~/types/frontend/impl/admin";

interface StatsGridProps {
    stats: AdminStats;
    role: AdminRole;
}

export function StatsGrid({ stats, role }: StatsGridProps) {
    const showUsersCard = canManageUsers(role) && stats.users;

    return (
        <div className={cn("grid grid-cols-1 gap-4 sm:grid-cols-2", showUsersCard ? "lg:grid-cols-4" : "lg:grid-cols-3")}>
            {/* Total Users - only visible to super_admin */}
            {showUsersCard && stats.users && (
                <div className="rounded-xl border bg-card p-4 text-card-foreground">
                    <div className="mb-3 flex items-center justify-between">
                        <span className="font-medium text-sm">Total Users</span>
                        <Users className="size-4 text-muted-foreground" />
                    </div>
                    <div className="rounded-lg border bg-secondary/50 p-4">
                        <div className="flex items-center justify-between">
                            <span className="font-medium text-2xl tracking-tight sm:text-3xl">{stats.users.total}</span>
                            <div className="flex items-center gap-3">
                                <div className="h-9 w-px bg-border" />
                                <div
                                    className={cn("flex items-center gap-1.5", "text-green-400")}
                                    style={{
                                        textShadow: "0 1px 6px rgba(68, 255, 118, 0.25)",
                                    }}
                                >
                                    <MoveUpRight className="size-3.5" />
                                    <span className="font-medium text-sm">12%</span>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div className="mt-3 flex flex-col gap-2">
                        <Separator />
                        <p className="text-muted-foreground text-xs">
                            {stats.users.byRole.super_admin} admins, {stats.users.byRole.tier_list_editor} editors
                        </p>
                    </div>
                </div>
            )}

            <div className="rounded-xl border bg-card p-4 text-card-foreground">
                <div className="mb-3 flex items-center justify-between">
                    <span className="font-medium text-sm">Tier Lists</span>
                    <LayoutList className="size-4 text-muted-foreground" />
                </div>
                <div className="rounded-lg border bg-secondary/50 p-4">
                    <div className="flex items-center justify-between">
                        <span className="font-medium text-2xl tracking-tight sm:text-3xl">{stats.tierLists.total}</span>
                        <div className="flex items-center gap-3">
                            <div className="h-9 w-px bg-border" />
                            <div
                                className={cn("flex items-center gap-1.5", "text-pink-400")}
                                style={{
                                    textShadow: "0 1px 6px rgba(255, 68, 193, 0.25)",
                                }}
                            >
                                <MoveDownRight className="size-3.5" />
                                <span className="font-medium text-sm">23%</span>
                            </div>
                        </div>
                    </div>
                </div>
                <div className="mt-3 flex flex-col gap-2">
                    <Separator />
                    <p className="text-muted-foreground text-xs">{stats.tierLists.active} active</p>
                </div>
            </div>

            <div className="rounded-xl border bg-card p-4 text-card-foreground">
                <div className="mb-3 flex items-center justify-between">
                    <span className="font-medium text-sm">Total Versions</span>
                    <History className="size-4 text-muted-foreground" />
                </div>
                <div className="rounded-lg border bg-secondary/50 p-4">
                    <div className="flex items-center justify-between">
                        <span className="font-medium text-2xl tracking-tight sm:text-3xl">{stats.tierLists.totalVersions}</span>
                        <div className="flex items-center gap-3">
                            <div className="h-9 w-px bg-border" />
                            <div
                                className={cn("flex items-center gap-1.5", "text-green-400")}
                                style={{
                                    textShadow: "0 1px 6px rgba(68, 255, 118, 0.25)",
                                }}
                            >
                                <MoveUpRight className="size-3.5" />
                                <span className="font-medium text-sm">17%</span>
                            </div>
                        </div>
                    </div>
                </div>
                <div className="mt-3 flex flex-col gap-2">
                    <Separator />
                    <p className="text-muted-foreground text-xs">Published across all tier lists</p>
                </div>
            </div>

            <div className="rounded-xl border bg-card p-4 text-card-foreground">
                <div className="mb-3 flex items-center justify-between">
                    <span className="font-medium text-sm">Operator Placements</span>
                    <Activity className="size-4 text-muted-foreground" />
                </div>
                <div className="rounded-lg border bg-secondary/50 p-4">
                    <div className="flex items-center justify-between">
                        <span className="font-medium text-2xl tracking-tight sm:text-3xl">{stats.tierLists.totalPlacements}</span>
                        <div className="flex items-center gap-3">
                            <div className="h-9 w-px bg-border" />
                            <div
                                className={cn("flex items-center gap-1.5", "text-pink-400")}
                                style={{
                                    textShadow: "0 1px 6px rgba(255, 68, 193, 0.25)",
                                }}
                            >
                                <MoveDownRight className="size-3.5" />
                                <span className="font-medium text-sm">38%</span>
                            </div>
                        </div>
                    </div>
                </div>
                <div className="mt-3 flex flex-col gap-2">
                    <Separator />
                    <p className="text-muted-foreground text-xs">Total rankings across tier lists</p>
                </div>
            </div>
        </div>
    );
}
