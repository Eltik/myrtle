"use client";

import { useRouter } from "next/router";
import { useCallback, useEffect, useState } from "react";
import { AdminPanel } from "~/components/admin";
import { SEO } from "~/components/seo";
import { useAuth } from "~/hooks/use-auth";
import type { AdminRole } from "~/lib/permissions";
import { isAdminRole } from "~/lib/permissions";
import type { AdminStats } from "~/types/frontend/impl/admin";

export default function AdminPage() {
    const router = useRouter();
    const { user, loading, verify } = useAuth();
    const [role, setRole] = useState<string | null>(null);
    const [authorized, setAuthorized] = useState<boolean | null>(null);
    const [stats, setStats] = useState<AdminStats | null>(null);
    const [statsLoading, setStatsLoading] = useState(true);

    // Fetch admin stats
    const fetchStats = useCallback(async () => {
        setStatsLoading(true);
        try {
            const res = await fetch("/api/admin/stats");
            const data = await res.json();

            if (data.success && data.data) {
                setStats(data.data);
            }
        } catch (error) {
            console.error("Failed to fetch admin stats:", error);
        } finally {
            setStatsLoading(false);
        }
    }, []);

    useEffect(() => {
        async function checkPermissions() {
            if (!user?.status) {
                setAuthorized(false);
                return;
            }

            const result = await verify();

            if (result.valid && isAdminRole(result.role)) {
                setRole(result.role);
                setAuthorized(true);
            } else {
                setAuthorized(false);
            }
        }

        if (!loading) {
            checkPermissions();
        }
    }, [user, loading, verify]);

    // Fetch stats once authorized
    useEffect(() => {
        if (authorized === true) {
            fetchStats();
        }
    }, [authorized, fetchStats]);

    // Redirect to 404 if not authorized
    useEffect(() => {
        if (authorized === false) {
            router.replace("/404");
        }
    }, [authorized, router]);

    // Loading state (show while checking auth or redirecting)
    if (loading || authorized === null || authorized === false) {
        return (
            <>
                <SEO description="myrtle.moe" noIndex path="/admin" title="Admin" />
                <div className="flex min-h-[50vh] items-center justify-center">
                    <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
                </div>
            </>
        );
    }

    // Safety check - should never happen given the auth flow above
    if (!user) {
        return null;
    }

    // Authorized admin view
    return (
        <>
            <SEO description="Admin panel for myrtle.moe" noIndex path="/admin" title="Admin" />
            <AdminPanel onRefresh={fetchStats} role={role as AdminRole} stats={stats} statsLoading={statsLoading} user={user} />
        </>
    );
}
