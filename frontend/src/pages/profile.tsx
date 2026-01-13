"use client";

import { useRouter } from "next/router";
import { useEffect } from "react";
import { SEO } from "~/components/seo";
import { useAuth } from "~/hooks/use-auth";

export default function ProfilePage() {
    const router = useRouter();
    const { user, loading } = useAuth();

    useEffect(() => {
        if (loading) return;

        if (user?.status?.uid) {
            router.replace(`/user/${user.status.uid}`);
        }
    }, [user, loading, router]);

    // Show loading state while checking auth
    if (loading) {
        return (
            <>
                <SEO description="View your Arknights profile on myrtle.moe" noIndex path="/profile" title="Profile" />
                <div className="container mx-auto flex min-h-[50vh] items-center justify-center p-4">
                    <div className="text-center">
                        <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
                    </div>
                </div>
            </>
        );
    }

    // If not logged in, show 404-style page
    if (!user?.status?.uid) {
        return (
            <>
                <SEO description="Profile not found. You need to be logged in to view your profile." noIndex path="/profile" title="Profile Not Found" />
                <div className="container mx-auto flex min-h-[50vh] items-center justify-center p-4">
                    <div className="text-center">
                        <h1 className="mb-4 font-bold text-4xl">Profile Not Found</h1>
                        <p className="text-muted-foreground">You need to be logged in to view your profile.</p>
                    </div>
                </div>
            </>
        );
    }

    // Redirecting state (shouldn't be visible for long)
    return (
        <>
            <SEO description="Redirecting to your profile" noIndex path="/profile" title="Redirecting..." />
            <div className="container mx-auto flex min-h-[50vh] items-center justify-center p-4">
                <div className="text-center">
                    <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
                </div>
            </div>
        </>
    );
}
