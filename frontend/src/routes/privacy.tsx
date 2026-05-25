import { createFileRoute } from "@tanstack/react-router";
import { PrivacyPage } from "#/components/legal/PrivacyPage";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/privacy")({
    component: PrivacyPage,
    head: () => {
        const { meta, links } = seo({
            title: "Privacy Policy",
            description: "Your privacy matters. Here's how Myrtle protects and handles your personal information.",
            path: "/privacy",
            image: defaultOgURL("privacy"),
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});
