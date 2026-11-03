import { createFileRoute } from "@tanstack/react-router";
import { StageList } from "#/components/stages/list/StageList";
import { RouteError } from "#/components/ui/route-error";
import { stageIndexQueryOptions } from "#/lib/api/stages";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/stages")({
    component: RouteComponent,
    errorComponent: RouteError,
    loader: async ({ context }) => {
        await context.queryClient.ensureQueryData(stageIndexQueryOptions(context.i18n.gamedataServer));
    },
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("stages.title"),
            description: t("stages.description"),
            path: "/stages",
            image: defaultOgURL("stages", match.context.i18n),
            locale: match.context.i18n?.locale,
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    return <StageList />;
}
