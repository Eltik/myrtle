import { createFileRoute } from "@tanstack/react-router";
import { ReleasePlanner } from "#/components/tools/release/ReleasePlanner";
import { RouteError } from "#/components/ui/route-error";
import { releaseEventsQueryOptions } from "#/lib/api/release";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/tools/release")({
    component: RouteComponent,
    errorComponent: RouteError,
    loader: ({ context: { queryClient } }) => queryClient.ensureQueryData(releaseEventsQueryOptions()),
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("toolsRelease.title"),
            description: t("toolsRelease.description"),
            path: "/tools/release",
            image: defaultOgURL("tools-release", match.context.i18n),
            locale: match.context.i18n?.locale,
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    return <ReleasePlanner />;
}
