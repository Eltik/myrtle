import { ClientOnly } from "@tanstack/react-router";
import { lazy, Suspense } from "react";
import { Skeleton } from "#/components/ui/skeleton";
import type { IChibiCharacter, IChibiSkin } from "#/lib/api/chibis";

interface IChibiViewerProps {
    chibi: IChibiCharacter | null;
    skin: IChibiSkin | null;
}

const LazyChibiViewer = lazy(() => import("./ChibiViewer").then((m) => ({ default: m.ChibiViewer })));

export function DynamicChibiViewer(props: IChibiViewerProps) {
    return (
        <ClientOnly fallback={<ChibiSkeleton />}>
            <Suspense fallback={<ChibiSkeleton />}>
                <LazyChibiViewer {...props} />
            </Suspense>
        </ClientOnly>
    );
}

function ChibiSkeleton() {
    return (
        <div className="w-full rounded-lg border border-border bg-card/30 p-3">
            <Skeleton className="mb-2 h-4 w-24" />
            <div className="mb-3 flex flex-wrap gap-2">
                <Skeleton className="h-8 w-22.5" />
                <Skeleton className="h-8 min-w-25 flex-1" />
            </div>
            <Skeleton className="h-45 w-full rounded-md" />
        </div>
    );
}
