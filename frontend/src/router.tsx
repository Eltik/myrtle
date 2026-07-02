import { createRouter as createTanStackRouter } from "@tanstack/react-router";
import { setupRouterSsrQueryIntegration } from "@tanstack/react-router-ssr-query";
import { NotFound } from "#/components/NotFound";
import { getContext } from "./integrations/tanstack-query/root-provider";
import { routeTree } from "./routeTree.gen";

export function getRouter() {
    const context = getContext();

    const router = createTanStackRouter({
        routeTree,
        context,
        scrollRestoration: true,
        defaultPreload: "intent",
        defaultPreloadStaleTime: 0,
        defaultNotFoundComponent: NotFound,
    });

    setupRouterSsrQueryIntegration({ router, queryClient: context.queryClient });
    if (typeof window === "undefined") {
        const dehydrate = router.options.dehydrate;
        router.options.dehydrate = async () => {
            const dehydrated = await dehydrate?.();
            router.serverSsr?.onRenderFinished(() => {
                context.queryClient.clear();
            });
            return dehydrated;
        };
    }

    return router;
}

declare module "@tanstack/react-router" {
    interface Register {
        router: ReturnType<typeof getRouter>;
    }
}
