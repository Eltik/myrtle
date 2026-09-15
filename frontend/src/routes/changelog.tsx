import { createFileRoute } from "@tanstack/react-router";
import { ChangelogError } from "#/components/changelog/ChangelogError";
import { ChangelogPage } from "#/components/changelog/ChangelogPage";
import { changelogQueryOptions } from "#/lib/api/changelog";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/changelog")({
    component: ChangelogPage,
    errorComponent: ChangelogError,
    loader: ({ context }) => context.queryClient.ensureQueryData(changelogQueryOptions()),
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("changelog.title"),
            description: t("changelog.description"),
            path: "/changelog",
            image: defaultOgURL("changelog", match.context.i18n),
            locale: match.context.i18n?.locale,
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});
