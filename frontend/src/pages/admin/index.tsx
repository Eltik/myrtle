"use client";

import { ShieldCheck } from "lucide-react";
import Head from "next/head";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { useAuth } from "~/hooks/use-auth";

type AdminRole = "super_admin" | "tier_list_admin";

const ADMIN_ROLES: AdminRole[] = ["super_admin", "tier_list_admin"];

function isAdminRole(role: string | undefined): role is AdminRole {
    return role !== undefined && ADMIN_ROLES.includes(role as AdminRole);
}

export default function AdminPage() {
    const router = useRouter();
    const { user, loading, verify } = useAuth();
    const [role, setRole] = useState<string | null>(null);
    const [authorized, setAuthorized] = useState<boolean | null>(null);

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
                <Head>
                    <title>Admin - myrtle.moe</title>
                    <meta content="Admin panel" name="description" />
                </Head>
                <div className="flex min-h-[50vh] items-center justify-center">
                    <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
                </div>
            </>
        );
    }

    // Authorized admin view
    return (
        <>
            <Head>
                <title>Admin - myrtle.moe</title>
                <meta content="Admin panel" name="description" />
            </Head>

            <div className="mx-auto max-w-4xl space-y-6">
                <div className="space-y-2">
                    <div className="flex items-center gap-3">
                        <ShieldCheck className="h-8 w-8 text-primary" />
                        <h1 className="font-bold text-3xl">Admin Panel</h1>
                    </div>
                    <p className="text-muted-foreground">
                        Logged in as <span className="font-medium">{user?.status.nickName}</span> with role <span className="font-mono text-primary">{role}</span>
                    </p>
                </div>

                <Card>
                    <CardHeader>
                        <CardTitle>Admin Dashboard</CardTitle>
                        <CardDescription>Administrative tools and management</CardDescription>
                    </CardHeader>
                    <CardContent>
                        <p className="text-muted-foreground text-sm">Admin functionality coming soon.</p>
                    </CardContent>
                </Card>
            </div>
        </>
    );
}
