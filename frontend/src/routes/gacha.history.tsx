import { createFileRoute } from "@tanstack/react-router";
import { HistoryPage } from "#/components/gacha/history/HistoryPage";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/gacha/history")({
    component: RouteComponent,
    head: () => {
        const { meta, links } = seo({
            title: "Gacha · My History",
            description: "Your personal gacha pull history, pity counters, and operator statistics.",
            path: "/gacha/history",
            image: defaultOgURL("gacha-history"),
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
