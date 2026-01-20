import dynamic from "next/dynamic";
import type { ForwardedRef } from "react";
import { Skeleton } from "~/components/ui/shadcn/skeleton";
import type { DpsChartHandle } from "./dps-chart";
import type { OperatorConfiguration } from "./types";

interface DynamicDpsChartProps {
    operators: OperatorConfiguration[];
    mode: "defense" | "resistance";
    ref?: ForwardedRef<DpsChartHandle>;
}

// Dynamic import of DpsChart component - splits Recharts into separate chunk
// Using a component that passes ref as prop since dynamic() supports ref forwarding in Next.js 13+
export const DynamicDpsChart = dynamic(() => import("./dps-chart").then((mod) => mod.DpsChart), {
    loading: () => (
        <div className="h-100 w-full space-y-4">
            <Skeleton className="h-6 w-32" />
            <Skeleton className="h-87.5 w-full rounded-lg" />
        </div>
    ),
    ssr: false,
}) as React.ForwardRefExoticComponent<DynamicDpsChartProps>;
