import { createFileRoute, redirect } from "@tanstack/react-router";
import { MyGrids } from "#/components/grids/MyGrids";
import { myGridsQueryOptions } from "#/lib/api/grids";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/_authed/grids_/my")({
    beforeLoad: ({ context, location }) => {
        if (!context.user) throw redirect({ to: "/", search: { auth: "1", next: location.href } });
    },
    component: MyGrids,
    loader: ({ context }) => context.queryClient.ensureQueryData(myGridsQueryOptions(context.user?.id ?? null, context.i18n.gamedataServer)),
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({ title: t("myGrids.title"), description: t("myGrids.description"), path: "/grids/my", image: defaultOgURL("grids", match.context.i18n), locale: match.context.i18n?.locale });
        return { meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, { name: "robots", content: "noindex,nofollow" }, ...meta], links };
    },
});
