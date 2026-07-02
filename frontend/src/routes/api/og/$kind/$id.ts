import { createFileRoute } from "@tanstack/react-router";
import { ogResponse, ogVersion } from "#/lib/og/impl/respond";

export const Route = createFileRoute("/api/og/$kind/$id")({
    server: {
        handlers: {
            GET: ({ params, request }) => ogResponse({ kind: params.kind, fetchId: params.id, version: ogVersion(request) }),
        },
    },
});
