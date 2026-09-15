import { createFileRoute } from "@tanstack/react-router";
import { OperatorPlanner } from "#/components/tools/planner/OperatorPlanner";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/tools/planner")({
    component: RouteComponent,
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("toolsPlanner.title"),
            description: t("toolsPlanner.description"),
            path: "/tools/planner",
            image: defaultOgURL("tools-planner", match.context.i18n),
            locale: match.context.i18n?.locale,
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
