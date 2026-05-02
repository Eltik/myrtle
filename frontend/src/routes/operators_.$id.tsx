import { createFileRoute } from "@tanstack/react-router";
import { OperatorDetail } from "#/components/operators/detail/Operators";
import { operatorQueryOptions } from "#/lib/api/operators";

export const Route = createFileRoute("/operators_/$id")({
    component: RouteComponent,
    errorComponent: RootErrorComponent,
    loader: ({ context, params }) => context.queryClient.ensureQueryData(operatorQueryOptions(params.id)),
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
