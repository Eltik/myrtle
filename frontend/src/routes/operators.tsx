import { OperatorsList } from "#/components/operators/Operators";
import { operatorsListQueryOptions } from "#/lib/api/operators";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/operators")({
    component: RouteComponent,
    errorComponent: RootErrorComponent,
    loader: ({ context }) => context.queryClient.ensureQueryData(operatorsListQueryOptions()),
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
