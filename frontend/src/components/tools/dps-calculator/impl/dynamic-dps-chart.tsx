import dynamic from "next/dynamic";
import type { RefObject } from "react";
import { Skeleton } from "~/components/ui/shadcn/skeleton";
import type { DpsChartHandle } from "./dps-chart";
import type { OperatorConfiguration } from "./types";

export interface DynamicDpsChartProps {
    operators: OperatorConfiguration[];
    mode: "defense" | "resistance";
    /** Pass ref as prop since Next.js dynamic() doesn't forward refs */
    chartRef?: RefObject<DpsChartHandle | null>;
}

// Dynamic import of DpsChart component - splits Recharts into separate chunk
// Note: We pass chartRef as a prop instead of using React's ref mechanism
// because Next.js dynamic() doesn't forward refs to the underlying component
export const DynamicDpsChart = dynamic(() => import("./dps-chart").then((mod) => mod.DpsChart), {
    loading: () => (
        <div className="h-100 w-full space-y-4">
            <Skeleton className="h-6 w-32" />
            <Skeleton className="h-87.5 w-full rounded-lg" />
        </div>
    ),
    ssr: false,
}) as React.ComponentType<DynamicDpsChartProps>;
