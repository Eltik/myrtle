import { createFileRoute } from "@tanstack/react-router";
import { OperatorPlanner } from "#/components/tools/planner/OperatorPlanner";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/tools/planner")({
    component: RouteComponent,
    head: () => {
        const { meta, links } = seo({
            title: "Operator Planner",
            description: "Plan your operator promotion, level, skill, and module targets.",
            path: "/tools/planner",
            image: defaultOgURL("tools-planner"),
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    return <OperatorPlanner />;
}
