import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { AlertTriangle } from "lucide-react";
import * as React from "react";
import { HpsCalculator } from "#/components/tools/hps/HpsCalculator";
import { Button } from "#/components/ui/button";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { hpsOperatorsQueryOptions } from "#/lib/api/hps";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export interface IHpsSearch {
    /** The whole setup as a share link (`encodeHpsShare`); rewritten as the page changes. */
    s?: string;
}

export const Route = createFileRoute("/tools/hps")({
    component: RouteComponent,
    errorComponent: HpsErrorComponent,
    validateSearch: (search: Record<string, unknown>): IHpsSearch => (typeof search.s === "string" && search.s.length > 0 ? { s: search.s } : {}),
    // Per-operator detail (skill names, module icons) is fetched when an operator
    // is picked, by `useOperatorDetail`; nothing operator-wide is needed to render.
    loader: ({ context: { queryClient } }) => queryClient.prefetchQuery(hpsOperatorsQueryOptions()),
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("toolsHps.title"),
            description: t("toolsHps.description"),
            path: "/tools/hps",
            image: defaultOgURL("tools-hps", match.context.i18n),
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
    // Replace, not push: the link follows every edit, and Back should leave the page, not undo a slider.
    const onShareParamChange = React.useCallback((s: string | undefined) => void navigate({ to: "/tools/hps", search: s ? { s } : {}, replace: true, resetScroll: false }), [navigate]);
    return <HpsCalculator shareParam={search.s} onShareParamChange={onShareParamChange} />;
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
                    <EmptyTitle>HPS Calculator failed to load</EmptyTitle>
                    <EmptyDescription>{message || "An unexpected error occurred while loading the HPS engine. Try again, or check the browser console for details."}</EmptyDescription>
                </EmptyHeader>
                <Button onClick={reset} variant="outline">
                    Retry
                </Button>
            </Empty>
        </div>
    );
}
