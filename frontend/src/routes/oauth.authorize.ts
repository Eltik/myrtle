import { createFileRoute } from "@tanstack/react-router";
import { packAuthorizeRequest } from "#/lib/oauth-request";

// The OAuth 2.0 authorization endpoint ("Sign in with myrtle.moe"): another
// site sends a player's browser here with `client_id`, `redirect_uri`,
// `state` and the rest in the query string.
//
// A server route on purpose, with no component. The request must reach the
// consent page exactly as it was sent, and the router's search parsing would
// rewrite it on the way; see `lib/oauth-request.ts`. So the raw query string
// is packed into one opaque value and the browser is sent on to the page.
export const Route = createFileRoute("/oauth/authorize")({
    server: {
        handlers: {
            GET: ({ request }) => {
                const url = new URL(request.url);
                // The path is rewritten in place, so a locale prefix carries over.
                const consent = url.pathname.replace(/\/authorize\/?$/, "/consent");
                return new Response(null, {
                    status: 302,
                    headers: {
                        Location: `${consent}?request=${packAuthorizeRequest(url.search)}`,
                        // Never cached: the next request may carry another state.
                        "Cache-Control": "no-store",
                    },
                });
            },
        },
    },
});
