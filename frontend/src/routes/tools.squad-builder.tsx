import { createFileRoute, useNavigate } from "@tanstack/react-router";
import * as React from "react";
import { hasShare, type IShareSearch, parseShareSearch } from "#/components/tools/squad-builder/model";
import { SquadBuilder } from "#/components/tools/squad-builder/SquadBuilder";
import { RouteError } from "#/components/ui/route-error";
import { resolveGamedataServer } from "#/lib/api/gamedata";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { squadRulesQueryOptions } from "#/lib/api/squads";
import { metaT } from "#/lib/meta";
import { defaultOgURL, ogURL } from "#/lib/og";
import { buildSquadOgData, parseSquadOgId, squadOgId } from "#/lib/og/impl/squad";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/tools/squad-builder")({
    component: RouteComponent,
    errorComponent: RouteError,
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
        // A share link unfurls as that squad, redrawn from the id alone (`squadOgId`);
        // no seat, or a link too long to carry, is the tool's preset card.
        const server = resolveGamedataServer(match.context.i18n.gamedataServer);
        const id = squadOgId(match.search, server);
        const parsed = id ? parseSquadOgId(id) : null;
        const image = id && parsed ? ogURL("squad", id, buildSquadOgData(parsed.draft, server)) : defaultOgURL("tools-squad-builder", match.context.i18n);
        const { meta, links } = seo({
            title: t("toolsSquadBuilder.title"),
            description: t("toolsSquadBuilder.description"),
            path: "/tools/squad-builder",
            image,
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
