import type { AuthUser } from "~/hooks/use-auth";
import type { AdminRole, AdminStats } from "~/types/frontend/admin";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../ui/shadcn/card";
import { Header } from "./impl/header";
import { StatsGrid } from "./impl/stats-grid";
import { UsersTable } from "./impl/users-table";

interface AdminPanelProps {
    user: AuthUser;
    role: AdminRole;
    stats: AdminStats | null;
    statsLoading: boolean;
    onRefresh: () => void;
}

export function AdminPanel({ user, role, stats, statsLoading, onRefresh }: AdminPanelProps) {
    return (
        <div className="mx-auto max-w-6xl space-y-6 rounded-md border border-border bg-card p-5">
            {/* Header */}
            <Header onRefresh={onRefresh} role={role} statsLoading={statsLoading} user={user} />

            {/* Stats Grid */}
            {statsLoading ? (
                <div className="flex min-h-[200px] items-center justify-center">
                    <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
                </div>
            ) : stats ? (
                <StatsGrid stats={stats} />
            ) : (
                <Card>
                    <CardHeader>
                        <CardTitle>No Data Available</CardTitle>
                        <CardDescription>Backend /admin/stats endpoint needs to be implemented to display statistics.</CardDescription>
                    </CardHeader>
                </Card>
            )}

            {/* Users Table */}
            {stats && stats.users.recentUsers.length > 0 && <UsersTable loading={statsLoading} onRefresh={onRefresh} users={stats.users.recentUsers} />}

            {/* Recent Activity */}
            {stats && stats.recentActivity.length > 0 && (
                <Card>
                    <CardHeader>
                        <CardTitle>Recent Activity</CardTitle>
                        <CardDescription>Latest changes across all tier lists</CardDescription>
                    </CardHeader>
                    <CardContent>
                        <div className="space-y-2">
                            {stats.recentActivity.slice(0, 10).map((activity) => (
                                <div className="flex items-center justify-between rounded border p-2 text-sm" key={activity.id}>
                                    <div>
                                        <span className="font-medium">{activity.changeType}</span>
                                        {activity.operatorName && <span className="ml-2 text-muted-foreground">on {activity.operatorName}</span>}
                                        <span className="ml-2 text-muted-foreground">in {activity.tierListName}</span>
                                    </div>
                                    <div className="text-muted-foreground text-xs">
                                        {activity.changedByNickname && <span>{activity.changedByNickname} · </span>}
                                        {new Date(activity.changedAt).toLocaleDateString()}
                                    </div>
                                </div>
                            ))}
                        </div>
                    </CardContent>
                </Card>
            )}
        </div>
    );
}
