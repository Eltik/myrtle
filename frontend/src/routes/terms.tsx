import { createFileRoute } from "@tanstack/react-router";
import { TermsPage } from "#/components/legal/TermsPage";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/terms")({
    component: TermsPage,
    head: () => {
        const { meta, links } = seo({
            title: "Terms of Service",
            description: "Legal agreement governing your use of Myrtle.",
            path: "/terms",
            image: defaultOgURL("terms"),
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});
