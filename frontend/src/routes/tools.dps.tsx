import { createFileRoute } from "@tanstack/react-router";
import { AlertTriangle } from "lucide-react";
import { DpsCalculator } from "#/components/tools/dps/DpsCalculator";
import { Button } from "#/components/ui/button";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { dpsOperatorsQueryOptions } from "#/lib/api/dps";
import { operatorsListQueryOptions } from "#/lib/api/operators";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/tools/dps")({
    component: RouteComponent,
    errorComponent: DpsErrorComponent,
    loader: ({ context: { queryClient } }) => Promise.all([queryClient.prefetchQuery(dpsOperatorsQueryOptions()), queryClient.prefetchQuery(operatorsListQueryOptions())]),
    head: () => {
        const { meta, links } = seo({
            title: "DPS Calculator",
            description: "Compare Arknights operator DPS across varying enemy DEF, RES, or target count. Configure skills, modules, buffs, and conditionals.",
            path: "/tools/dps",
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    return <DpsCalculator />;
}

function DpsErrorComponent({ error, reset }: { error: unknown; reset: () => void }) {
    const message = error instanceof Error ? error.message : String(error);
    return (
        <div className="relative z-1 mx-auto w-[min(640px,calc(100%-2rem))] py-20">
            <Empty>
                <EmptyHeader>
                    <EmptyMedia variant="icon">
                        <AlertTriangle className="text-destructive" />
                    </EmptyMedia>
                    <EmptyTitle>DPS calculator failed to load</EmptyTitle>
                    <EmptyDescription>{message || "An unexpected error occurred while loading the DPS engine. Try again, or check the browser console for details."}</EmptyDescription>
                </EmptyHeader>
                <Button onClick={reset} variant="outline">
                    Retry
                </Button>
            </Empty>
        </div>
    );
}
