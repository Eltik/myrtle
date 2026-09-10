import { createFileRoute } from "@tanstack/react-router";
import { OperatorDetail } from "#/components/operators/detail/Operators";
import { operatorBuildStatsQueryOptions, operatorQueryOptions, operatorsIndexQueryOptions } from "#/lib/api/operators";
import { ogURL, warmOg } from "#/lib/og/impl/url";
import { seo } from "#/lib/seo";
import { formatProfession, formatSubProfession } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";

function buildOgData(operator: IOperatorListItem) {
    const rarityNum = Number(String(operator.rarity).replace("TIER_", "")) || 1;
    return {
        name: operator.name,
        appellation: operator.appellation ?? "",
        profession: operator.profession,
        rarity: rarityNum as 1 | 2 | 3 | 4 | 5 | 6,
        // Carried so the OG cache key differs between the CN and Global art
        // trees; `/operators/{id}` tags every operator with the server it
        // resolved on, CN-exclusive ones included.
        server: operator.server,
    };
}

export const Route = createFileRoute("/operators_/$id")({
    component: RouteComponent,
    errorComponent: RootErrorComponent,
    loader: async ({ context, params }) => {
        const operator = await context.queryClient.ensureQueryData(operatorQueryOptions(params.id));
        if (operator) {
            // Community build stats decide which skill and module the tabs open
            // on, so they are prefetched rather than fetched from the tab: a
            // late arrival would visibly re-select in front of the reader.
            // Fire-and-forget, because a missing aggregate is a fallback, not
            // an error, and must never block the page.
            void context.queryClient.prefetchQuery(operatorBuildStatsQueryOptions(params.id));
            warmOg("operator", params.id, buildOgData(operator));
            // For operators with alternate forms (Amiya), preload the index so
            // the form switcher renders in SSR without a hydration flash.
            if ((operator.tmplIds?.length ?? 0) >= 2) {
                await context.queryClient.ensureQueryData(operatorsIndexQueryOptions());
            }
        }
        return operator;
    },
    head: ({ loaderData, params }) => {
        if (!loaderData) return seo({ title: "Operator", path: `/operators/${params.id}` });
        const ogData = buildOgData(loaderData);
        return seo({
            title: loaderData.name,
            description: `${ogData.rarity}★ ${formatProfession(loaderData.profession)} • ${formatSubProfession(loaderData.subProfessionId ?? "")}`.trim(),
            image: ogURL("operator", params.id, ogData),
            path: `/operators/${params.id}`,
            type: "profile",
            preloadImage: true,
        });
    },
});

function RootErrorComponent({ error }: { error: unknown }) {
    console.error("Router error:", error);
    if (error instanceof Error) {
        console.error(error.stack);
    }

    return (
        <div style={{ padding: 20 }}>
            <h1>Something went wrong</h1>
            <pre style={{ color: "red" }}>{error instanceof Error ? error.message : JSON.stringify(error, null, 2)}</pre>
            <pre>{error instanceof Error ? error.stack : null}</pre>
        </div>
    );
}

function RouteComponent() {
    return <OperatorDetail />;
}
