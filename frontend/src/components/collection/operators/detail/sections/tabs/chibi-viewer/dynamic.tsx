import dynamic from "next/dynamic";
import { Skeleton } from "~/components/ui/shadcn/skeleton";

export const DynamicChibiViewer = dynamic(() => import("./index").then((mod) => mod.ChibiViewer), {
    loading: () => (
        <div className="w-full rounded-lg border border-border bg-card/30 p-3">
            <Skeleton className="mb-2 h-4 w-24" />
            <div className="mb-3 flex flex-wrap gap-2">
                <Skeleton className="h-8 w-22.5" />
                <Skeleton className="h-8 min-w-25 flex-1" />
            </div>
            <Skeleton className="h-45 w-full rounded-md" />
        </div>
    ),
    ssr: false,
});
