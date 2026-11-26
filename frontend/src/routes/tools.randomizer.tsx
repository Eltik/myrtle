import { createFileRoute } from "@tanstack/react-router";
import { Randomizer } from "#/components/tools/randomizer/Randomizer";
import { RouteError } from "#/components/ui/route-error";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { stagesQueryOptions, zonesQueryOptions } from "#/lib/api/stages";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/tools/randomizer")({
    component: RouteComponent,
    errorComponent: RouteError,
    loader: ({ context: { queryClient, i18n } }) => Promise.all([queryClient.prefetchQuery(operatorsIndexQueryOptions(i18n.gamedataServer)), queryClient.prefetchQuery(stagesQueryOptions(i18n.gamedataServer)), queryClient.prefetchQuery(zonesQueryOptions(i18n.gamedataServer))]),
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("toolsRandomizer.title"),
            description: t("toolsRandomizer.description"),
            path: "/tools/randomizer",
            image: defaultOgURL("tools-randomizer", match.context.i18n),
            locale: match.context.i18n?.locale,
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    return <Randomizer />;
}
