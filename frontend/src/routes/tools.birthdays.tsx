import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { AlertTriangle } from "lucide-react";
import { Birthdays } from "#/components/tools/birthdays/Birthdays";
import { Button } from "#/components/ui/button";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { operatorsListQueryOptions } from "#/lib/api/operators";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/tools/birthdays")({
    component: RouteComponent,
    errorComponent: BirthdaysErrorComponent,
    loader: ({ context: { queryClient } }) => queryClient.ensureQueryData(operatorsListQueryOptions()),
    head: () => {
        const { meta, links } = seo({
            title: "Birthdays",
            description: "View and track Arknights operator birthdays.",
            path: "/tools/birthdays",
            image: defaultOgURL("tools-birthdays"),
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    const { data: operators = [] } = useQuery(operatorsListQueryOptions());
    return <Birthdays operators={operators} />;
}

function BirthdaysErrorComponent({ error, reset }: { error: unknown; reset: () => void }) {
    const message = error instanceof Error ? error.message : String(error);
    return (
        <div className="relative z-1 mx-auto w-[min(640px,calc(100%-2rem))] py-20">
            <Empty>
                <EmptyHeader>
                    <EmptyMedia variant="icon">
                        <AlertTriangle className="text-destructive" />
                    </EmptyMedia>
                    <EmptyTitle>Birthdays failed to load</EmptyTitle>
                    <EmptyDescription>{message || "An unexpected error occurred while loading birthday data."}</EmptyDescription>
                </EmptyHeader>
                <Button onClick={reset} variant="outline">
                    Retry
                </Button>
            </Empty>
        </div>
    );
}
