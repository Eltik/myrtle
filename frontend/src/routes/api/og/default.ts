import { createFileRoute } from "@tanstack/react-router";
import { DEFAULT_OG_ID } from "#/lib/og/impl/registry";
import { ogResponse, ogVersion } from "#/lib/og/impl/respond";

export const Route = createFileRoute("/api/og/default")({
    server: {
        handlers: {
            GET: ({ request }) => ogResponse({ kind: "default", fetchId: DEFAULT_OG_ID, version: ogVersion(request) }),
        },
    },
});
