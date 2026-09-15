import { createFileRoute } from "@tanstack/react-router";
import { TermsPage } from "#/components/legal/TermsPage";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/terms")({
    component: TermsPage,
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("terms.title"),
            description: t("terms.description"),
            path: "/terms",
            image: defaultOgURL("terms", match.context.i18n),
            locale: match.context.i18n?.locale,
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});
