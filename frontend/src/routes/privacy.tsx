import { createFileRoute } from "@tanstack/react-router";
import { PrivacyPage } from "#/components/legal/PrivacyPage";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/privacy")({
    component: PrivacyPage,
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("privacy.title"),
            description: t("privacy.description"),
            path: "/privacy",
            image: defaultOgURL("privacy", match.context.i18n),
            locale: match.context.i18n?.locale,
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});
