import { createFileRoute } from "@tanstack/react-router";
import { AlertTriangle } from "lucide-react";
import { ReleasePlanner } from "#/components/tools/release/ReleasePlanner";
import { Button } from "#/components/ui/button";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { releaseEventsQueryOptions } from "#/lib/api/release";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/tools/release")({
    component: RouteComponent,
    errorComponent: ReleaseErrorComponent,
    loader: ({ context: { queryClient } }) => queryClient.ensureQueryData(releaseEventsQueryOptions()),
    head: () => {
        const { meta, links } = seo({
            title: "Release Planner",
            description: "When CN events, skins, and banners land on EN: confirmed, announced, or estimated with a band.",
            path: "/tools/release",
            image: defaultOgURL("tools-release"),
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    return <ReleasePlanner />;
}

function ReleaseErrorComponent({ error, reset }: { error: unknown; reset: () => void }) {
    const message = error instanceof Error ? error.message : String(error);
    return (
        <div className="relative z-1 mx-auto w-[min(640px,calc(100%-2rem))] py-20">
            <Empty>
                <EmptyHeader>
                    <EmptyMedia variant="icon">
                        <AlertTriangle className="text-destructive" />
                    </EmptyMedia>
                    <EmptyTitle>Release Planner failed to load</EmptyTitle>
                    <EmptyDescription>{message || "An unexpected error occurred while loading release data."}</EmptyDescription>
                </EmptyHeader>
                <Button onClick={reset} variant="outline">
                    Retry
                </Button>
            </Empty>
        </div>
    );
}
