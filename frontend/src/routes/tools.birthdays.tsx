import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Birthdays } from "#/components/tools/birthdays/Birthdays";
import { RouteError } from "#/components/ui/route-error";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { useGamedataServer } from "#/lib/i18n";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/tools/birthdays")({
    component: RouteComponent,
    errorComponent: RouteError,
    loader: ({ context: { queryClient, i18n } }) => queryClient.ensureQueryData(operatorsIndexQueryOptions(i18n.gamedataServer)),
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("toolsBirthdays.title"),
            description: t("toolsBirthdays.description"),
            path: "/tools/birthdays",
            image: defaultOgURL("tools-birthdays", match.context.i18n),
            locale: match.context.i18n?.locale,
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    const server = useGamedataServer();
    const { data: operators = [] } = useQuery(operatorsIndexQueryOptions(server));
    return <Birthdays operators={operators} />;
}
