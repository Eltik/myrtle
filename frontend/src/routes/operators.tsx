import { createFileRoute } from "@tanstack/react-router";
import { OperatorsList } from "#/components/operators/list/Operators";
import { operatorNotesListQueryOptions } from "#/lib/api/operator-notes";
import { operatorsListQueryOptions } from "#/lib/api/operators";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/operators")({
    component: RouteComponent,
    errorComponent: RootErrorComponent,
    loader: ({ context }) => Promise.all([context.queryClient.ensureQueryData(operatorsListQueryOptions()), context.queryClient.prefetchQuery(operatorNotesListQueryOptions())]),
    head: () => {
        const { meta, links } = seo({
            title: "Operators",
            description: "View all operators released in Arknights.",
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RootErrorComponent({ error }: { error: unknown }) {
    console.error("Router error:", error);

    return (
        <div style={{ padding: 20 }}>
            <h1>Something went wrong</h1>
            <pre style={{ color: "red" }}>{error instanceof Error ? error.message : JSON.stringify(error, null, 2)}</pre>
            <pre>{error instanceof Error ? error.stack : null}</pre>
        </div>
    );
}

function RouteComponent() {
    return <OperatorsList />;
}
