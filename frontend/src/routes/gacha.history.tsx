import { createFileRoute } from "@tanstack/react-router";
import { HistoryPage } from "#/components/gacha/history/HistoryPage";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/gacha/history")({
    component: RouteComponent,
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("gachaHistory.title"),
            description: t("gachaHistory.description"),
            path: "/gacha/history",
            image: defaultOgURL("gacha-history", match.context.i18n),
            locale: match.context.i18n?.locale,
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    return <HistoryPage />;
}
