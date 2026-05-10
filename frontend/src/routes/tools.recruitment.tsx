import { createFileRoute } from "@tanstack/react-router";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/tools/recruitment")({
    component: RouteComponent,
    head: () => {
        const { meta, links } = seo({
            title: "Recruitment Calculator",
            description: "Calculate optimal tag combinations for Arknights recruitment. Find guaranteed 5-star and 6-star operators with the best tag combos.",
            path: "/tools/recruitment",
            image: defaultOgURL("tools-recruitment"),
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    return null;
}
