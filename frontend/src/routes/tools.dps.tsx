import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { AlertTriangle } from "lucide-react";
import * as React from "react";
import { DpsCalculator } from "#/components/tools/dps/DpsCalculator";
import { Button } from "#/components/ui/button";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { dpsOperatorsQueryOptions } from "#/lib/api/dps";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export interface IDpsSearch {
    /** The Squad Builder's hand-off (`encodeDpsImport`); read once and dropped. */
    import?: string;
    /** The whole setup as a share link (`encodeDpsShare`); rewritten as the page changes. */
    s?: string;
}

export const Route = createFileRoute("/tools/dps")({
    component: RouteComponent,
    errorComponent: DpsErrorComponent,
    validateSearch: (search: Record<string, unknown>): IDpsSearch => ({
        ...(typeof search.import === "string" && search.import.length > 0 ? { import: search.import } : {}),
        ...(typeof search.s === "string" && search.s.length > 0 ? { s: search.s } : {}),
    }),
    // Per-operator detail (skill names, module icons) is fetched when an operator
    // is picked, by `useOperatorDetail`; nothing operator-wide is needed to render.
    loader: ({ context: { queryClient } }) => queryClient.prefetchQuery(dpsOperatorsQueryOptions()),
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("toolsDps.title"),
            description: t("toolsDps.description"),
            path: "/tools/dps",
            image: defaultOgURL("tools-dps", match.context.i18n),
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
    // Dropped with a replace once the page has read it, so a reload does not add the cards again.
    const onImportConsumed = React.useCallback(() => void navigate({ to: "/tools/dps", search: (prev) => ({ s: prev.s }), replace: true, resetScroll: false }), [navigate]);
    // Replace, not push: the link follows every edit, and Back should leave the page, not undo a slider.
    const onShareParamChange = React.useCallback((s: string | undefined) => void navigate({ to: "/tools/dps", search: (prev) => ({ import: prev.import, s }), replace: true, resetScroll: false }), [navigate]);
    return <DpsCalculator importParam={search.import} onImportConsumed={onImportConsumed} shareParam={search.s} onShareParamChange={onShareParamChange} />;
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
                    <EmptyTitle>DPS Calculator failed to load</EmptyTitle>
                    <EmptyDescription>{message || "An unexpected error occurred while loading the DPS engine. Try again, or check the browser console for details."}</EmptyDescription>
                </EmptyHeader>
                <Button onClick={reset} variant="outline">
                    Retry
                </Button>
            </Empty>
        </div>
    );
}
