import { createFileRoute } from "@tanstack/react-router";
import { DEFAULT_OG_ID } from "#/lib/og/impl/registry";
import { ogResponse } from "#/lib/og/impl/respond";

export const Route = createFileRoute("/api/og/default")({
    server: {
        handlers: {
            GET: () => ogResponse({ kind: "default", fetchId: DEFAULT_OG_ID }),
        },
    },
});
