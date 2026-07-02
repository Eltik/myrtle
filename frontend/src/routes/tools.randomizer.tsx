import { createFileRoute } from "@tanstack/react-router";
import { AlertTriangle } from "lucide-react";
import { Randomizer } from "#/components/tools/randomizer/Randomizer";
import { Button } from "#/components/ui/button";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { stagesQueryOptions, zonesQueryOptions } from "#/lib/api/stages";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/tools/randomizer")({
    component: RouteComponent,
    errorComponent: RandomizerErrorComponent,
    loader: ({ context: { queryClient } }) => Promise.all([queryClient.prefetchQuery(operatorsIndexQueryOptions()), queryClient.prefetchQuery(stagesQueryOptions()), queryClient.prefetchQuery(zonesQueryOptions())]),
    head: () => {
        const { meta, links } = seo({
            title: "Randomizer",
            description: "Roll a random Arknights stage, squad, and challenge modifier. Logged-in users can constrain rolls to stages they've cleared and operators they own.",
            path: "/tools/randomizer",
            image: defaultOgURL("tools-randomizer"),
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    return <Randomizer />;
}

function RandomizerErrorComponent({ error, reset }: { error: unknown; reset: () => void }) {
    const message = error instanceof Error ? error.message : String(error);
    return (
        <div className="relative z-1 mx-auto w-[min(640px,calc(100%-2rem))] py-20">
            <Empty>
                <EmptyHeader>
                    <EmptyMedia variant="icon">
                        <AlertTriangle className="text-destructive" />
                    </EmptyMedia>
                    <EmptyTitle>Randomizer failed to load</EmptyTitle>
                    <EmptyDescription>{message || "An unexpected error occurred while loading randomizer data."}</EmptyDescription>
                </EmptyHeader>
                <Button onClick={reset} variant="outline">
                    Retry
                </Button>
            </Empty>
        </div>
    );
}
