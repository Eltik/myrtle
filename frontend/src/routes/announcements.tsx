import { createFileRoute } from "@tanstack/react-router";
import { AnnouncementsPage } from "#/components/announcements/AnnouncementsPage";
import { validateAnnouncementsSearch } from "#/components/announcements/search";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/announcements")({
    component: AnnouncementsPage,
    validateSearch: validateAnnouncementsSearch,
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("announcements.title"),
            description: t("announcements.description"),
            path: "/announcements",
            image: defaultOgURL("home", match.context.i18n),
            locale: match.context.i18n?.locale,
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});
