import { createFileRoute, useNavigate } from "@tanstack/react-router";
import * as React from "react";
import { HpsCalculator } from "#/components/tools/hps/HpsCalculator";
import { RouteError } from "#/components/ui/route-error";
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
    errorComponent: RouteError,
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
