"use client";

import { useAuth } from "~/hooks/use-auth";
import { AuthRequiredWall } from "../ui/auth-required-wall";
import { ComponentFallback } from "../ui/component-fallback";

export default function ProtectedPageLayout({ children }: { children: React.ReactNode }) {
    const { user, loading } = useAuth();

    if (loading) {
        return <ComponentFallback />;
    }

    if (!user) {
        return <AuthRequiredWall />;
    }

    return <>{children}</>;
}
