import { createFileRoute } from "@tanstack/react-router";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/login")({
    validateSearch: (s) => ({ redirect: (s.redirect as string) ?? "/" }),
    component: RouteComponent,
    head: () => {
        const { meta, links } = seo({
            title: "Log in to Myrtle",
            description: "Sign in to track your roster, gacha history, and tier lists.",
            path: "/login",
            image: defaultOgURL("login"),
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    return <div>Hello "/login"!</div>;
}
