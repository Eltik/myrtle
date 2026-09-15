import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { AlertTriangle } from "lucide-react";
import { Birthdays } from "#/components/tools/birthdays/Birthdays";
import { Button } from "#/components/ui/button";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { operatorsListQueryOptions } from "#/lib/api/operators";
import { useGamedataServer } from "#/lib/i18n";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/tools/birthdays")({
    component: RouteComponent,
    errorComponent: BirthdaysErrorComponent,
    loader: ({ context: { queryClient, i18n } }) => queryClient.ensureQueryData(operatorsListQueryOptions(i18n.gamedataServer)),
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("toolsBirthdays.title"),
            description: t("toolsBirthdays.description"),
            path: "/tools/birthdays",
            image: defaultOgURL("tools-birthdays", match.context.i18n),
            locale: match.context.i18n?.locale,
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    const server = useGamedataServer();
    const { data: operators = [] } = useQuery(operatorsListQueryOptions(server));
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
