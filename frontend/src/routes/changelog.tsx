import { createFileRoute } from "@tanstack/react-router";
import { ChangelogError } from "#/components/changelog/ChangelogError";
import { ChangelogPage } from "#/components/changelog/ChangelogPage";
import { changelogQueryOptions } from "#/lib/api/changelog";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/changelog")({
    component: ChangelogPage,
    errorComponent: ChangelogError,
    loader: ({ context }) => context.queryClient.ensureQueryData(changelogQueryOptions()),
    head: () => {
        const { meta, links } = seo({
            title: "Changelog",
            description: "Every commit shipped to myrtle.moe, pulled live from GitHub and grouped by day, week, and month.",
            path: "/changelog",
            image: defaultOgURL("changelog"),
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});
