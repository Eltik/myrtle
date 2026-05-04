import { createFileRoute } from "@tanstack/react-router";
import { OperatorDetail } from "#/components/operators/detail/Operators";
import { operatorQueryOptions } from "#/lib/api/operators";
import { ogURL } from "#/lib/og/impl/url";
import { seo } from "#/lib/seo";
import { formatProfession, formatSubProfession } from "#/lib/utils";

export const Route = createFileRoute("/operators_/$id")({
    component: RouteComponent,
    errorComponent: RootErrorComponent,
    loader: ({ context, params }) => context.queryClient.ensureQueryData(operatorQueryOptions(params.id)),
    head: ({ loaderData, params }) => {
        if (!loaderData) return seo({ title: "Operator", path: `/operators/${params.id}` });
        const rarityNum = Number(String(loaderData.rarity).replace("TIER_", "")) || 1;
        const ogData = {
            name: loaderData.name,
            appellation: loaderData.appellation ?? "",
            profession: loaderData.profession,
            rarity: rarityNum as 1 | 2 | 3 | 4 | 5 | 6,
        };
        return seo({
            title: loaderData.name,
            description: `${rarityNum}★ ${formatProfession(loaderData.profession)} • ${formatSubProfession(loaderData.subProfessionId ?? "")}`.trim(),
            image: ogURL("operator", params.id, ogData),
            path: `/operators/${params.id}`,
            type: "profile",
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
