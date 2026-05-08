import { createFileRoute } from "@tanstack/react-router";
import { OperatorDetail } from "#/components/operators/detail/Operators";
import { operatorQueryOptions } from "#/lib/api/operators";
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
    };
}

export const Route = createFileRoute("/operators_/$id")({
    component: RouteComponent,
    errorComponent: RootErrorComponent,
    loader: async ({ context, params }) => {
        const operator = await context.queryClient.ensureQueryData(operatorQueryOptions(params.id));
        if (operator) warmOg("operator", params.id, buildOgData(operator));
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
