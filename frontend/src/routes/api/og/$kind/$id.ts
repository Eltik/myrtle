import { createFileRoute } from "@tanstack/react-router";
import { ogResponse } from "#/lib/og/impl/respond";

export const Route = createFileRoute("/api/og/$kind/$id")({
    server: {
        handlers: {
            GET: ({ params }) => ogResponse({ kind: params.kind, fetchId: params.id }),
        },
    },
});
