import { createFileRoute } from "@tanstack/react-router";
import { AlertTriangle } from "lucide-react";
import { HpsCalculator } from "#/components/tools/hps/HpsCalculator";
import { Button } from "#/components/ui/button";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { hpsOperatorsQueryOptions } from "#/lib/api/hps";
import { operatorsListQueryOptions } from "#/lib/api/operators";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/tools/hps")({
    component: RouteComponent,
    errorComponent: HpsErrorComponent,
    loader: ({ context: { queryClient } }) => Promise.all([queryClient.prefetchQuery(hpsOperatorsQueryOptions()), queryClient.prefetchQuery(operatorsListQueryOptions())]),
    head: () => {
        const { meta, links } = seo({
            title: "HPS Calculator",
            description: "Compare Arknights healer HPS across target counts and team buffs. Configure skills, modules, buffs, and conditionals.",
            path: "/tools/hps",
            image: defaultOgURL("tools-hps"),
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    return <HpsCalculator />;
}

function HpsErrorComponent({ error, reset }: { error: unknown; reset: () => void }) {
    const message = error instanceof Error ? error.message : String(error);
    return (
        <div className="relative z-1 mx-auto w-[min(640px,calc(100%-2rem))] py-20">
            <Empty>
                <EmptyHeader>
                    <EmptyMedia variant="icon">
                        <AlertTriangle className="text-destructive" />
                    </EmptyMedia>
                    <EmptyTitle>HPS calculator failed to load</EmptyTitle>
                    <EmptyDescription>{message || "An unexpected error occurred while loading the HPS engine. Try again, or check the browser console for details."}</EmptyDescription>
                </EmptyHeader>
                <Button onClick={reset} variant="outline">
                    Retry
                </Button>
            </Empty>
        </div>
    );
}
