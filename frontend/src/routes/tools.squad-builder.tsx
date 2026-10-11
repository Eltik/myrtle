import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { AlertTriangle } from "lucide-react";
import * as React from "react";
import { hasShare, type IShareSearch, parseShareSearch } from "#/components/tools/squad-builder/model";
import { SquadBuilder } from "#/components/tools/squad-builder/SquadBuilder";
import { Button } from "#/components/ui/button";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { squadRulesQueryOptions } from "#/lib/api/squads";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/tools/squad-builder")({
    component: RouteComponent,
    errorComponent: SquadBuilderErrorComponent,
    // The share link (`encodeShare`). The page reads it once and drops it with
    // a replace, so a reload does not reopen the shared squad over the player's own.
    validateSearch: (search: Record<string, unknown>): IShareSearch => parseShareSearch(search),
    // Started in the browser and not awaited: on a link hover (the router preloads on
    // intent) or alongside the route chunk. Awaited on the server, the 645,037-byte index
    // was dehydrated into the HTML, a 785,265-byte document for a page whose first view
    // (twelve empty seats) needs none of it.
    loader: ({ context: { queryClient, i18n } }) => {
        if (typeof window === "undefined") return;
        void queryClient.prefetchQuery(operatorsIndexQueryOptions(i18n.gamedataServer));
        void queryClient.prefetchQuery(squadRulesQueryOptions(i18n.gamedataServer));
    },
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("toolsSquadBuilder.title"),
            description: t("toolsSquadBuilder.description"),
            path: "/tools/squad-builder",
            image: defaultOgURL("tools-squad-builder", match.context.i18n),
            locale: match.context.i18n?.locale,
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    const search = Route.useSearch();
    const navigate = useNavigate();
    const share = hasShare(search as Record<string, unknown>) ? search : null;
    const onShareConsumed = React.useCallback(() => void navigate({ to: "/tools/squad-builder", search: {}, replace: true }), [navigate]);
    return <SquadBuilder share={share} onShareConsumed={onShareConsumed} />;
}

function SquadBuilderErrorComponent({ error, reset }: { error: unknown; reset: () => void }) {
    const message = error instanceof Error ? error.message : String(error);
    return (
        <div className="relative z-1 mx-auto w-[min(640px,calc(100%-2rem))] py-20">
            <Empty>
                <EmptyHeader>
                    <EmptyMedia variant="icon">
                        <AlertTriangle className="text-destructive" />
                    </EmptyMedia>
                    <EmptyTitle>Squad Builder failed to load</EmptyTitle>
                    <EmptyDescription>{message || "An unexpected error occurred while loading the Squad Builder. Try again, or check the browser console for details."}</EmptyDescription>
                </EmptyHeader>
                <Button onClick={reset} variant="outline">
                    Retry
                </Button>
            </Empty>
        </div>
    );
}
