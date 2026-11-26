import { createFileRoute } from "@tanstack/react-router";
import { AuthorizePage } from "#/components/oauth/AuthorizePage";
import { metaT } from "#/lib/meta";
import { unpackAuthorizeRequest } from "#/lib/oauth-request";
import { seo } from "#/lib/seo";

// The consent page of "Sign in with myrtle.moe". Players arrive from
// `/oauth/authorize`, which packs the other site's request into `request`
// (see `lib/oauth-request.ts` for why it is not plain search params).
//
// Not auth-gated by a redirect: a signed-out visitor signs in on this page
// and stays on it, request intact.
export const Route = createFileRoute("/oauth/consent")({
    validateSearch: (search: Record<string, unknown>) => ({ request: typeof search.request === "string" ? search.request : "" }),
    component: RouteComponent,
    // A consent screen must never render inside another site's frame, where a
    // click on something else could be steered onto "Authorize".
    headers: () => ({ "X-Frame-Options": "DENY", "Content-Security-Policy": "frame-ancestors 'none'" }),
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("oauthAuthorize.title"),
            description: t("oauthAuthorize.description"),
            path: "/oauth/consent",
            locale: match.context.i18n?.locale,
            noindex: true,
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    const { request } = Route.useSearch();
    return <AuthorizePage request={unpackAuthorizeRequest(request)} />;
}
