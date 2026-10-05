import { createFileRoute } from "@tanstack/react-router";
import { gridImageFilename, gridOgId } from "#/lib/og/impl/grid";
import { ogResponse, ogVersion } from "#/lib/og/impl/respond";

export const Route = createFileRoute("/api/grids/$slug/image")({
    server: {
        handlers: {
            GET: ({ params, request }) =>
                ogResponse({
                    kind: "grid-image",
                    // The viewer's server, so the PNG draws the same art the page shows; an unknown value falls back to the default.
                    fetchId: gridOgId(params.slug, new URL(request.url).searchParams.get("server") ?? undefined),
                    version: ogVersion(request),
                    attachmentFilename: gridImageFilename(params.slug),
                }),
        },
    },
});
