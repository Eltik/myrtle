import { createFileRoute } from "@tanstack/react-router";
import { ogResponse } from "#/lib/og/impl/respond";

function safeFilename(slug: string): string {
    const cleaned = slug.replace(/[^a-z0-9_-]+/gi, "-").replace(/^-+|-+$/g, "");
    const stem = cleaned || "tier-list";
    return /tier[-_]?list$/i.test(stem) ? `${stem}.png` : `${stem}-tier-list.png`;
}

export const Route = createFileRoute("/api/tier-lists/$slug/image")({
    server: {
        handlers: {
            GET: ({ params }) =>
                ogResponse({
                    kind: "tier-list-image",
                    fetchId: params.slug,
                    attachmentFilename: safeFilename(params.slug),
                }),
        },
    },
});
