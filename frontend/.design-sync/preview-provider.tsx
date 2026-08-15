// The context wrapper every preview card renders inside (`cfg.provider`), and a
// real export of the design-system bundle so a design built with these
// components can wrap itself the same way.
//
// Myrtle's components read four different contexts, and a component that needs
// one renders blank or throws without it:
//   - TanStack Router — anything with a <Link> or useRouterState (nav, cards,
//     tiles). RouterContextProvider supplies the router without mounting a
//     route tree, so children render as-is and <Link> resolves to a real <a>.
//   - TanStack Query — data containers. The client is configured to never retry
//     or refetch: their fetchers hit stubbed server functions that reject, and a
//     retrying client would keep the preview spinning.
//   - Toasts — Base UI's toast viewport, mounted by the app shell in __root.
//   - Command palette — useCommand() in the header, home page and 404.
//
// Theming needs no provider: it is a TanStack Store singleton plus a class on
// <html>, and styles.css defines the light palette on :root.

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterContextProvider, createMemoryHistory, createRootRoute, createRouter } from "@tanstack/react-router";
import type { ReactNode } from "react";
import { AnchoredToastProvider, ToastProvider } from "../src/components/ui/toast";
import { CommandProvider } from "../src/lib/command-context";

const previewRouter = createRouter({
    routeTree: createRootRoute(),
    history: createMemoryHistory({ initialEntries: ["/"] }),
});

const previewQueryClient = new QueryClient({
    defaultOptions: {
        queries: { retry: false, refetchOnWindowFocus: false, refetchOnMount: false, staleTime: Number.POSITIVE_INFINITY },
        mutations: { retry: false },
    },
});

export function DesignPreviewProvider({ children }: { children?: ReactNode }) {
    return (
        <QueryClientProvider client={previewQueryClient}>
            {/* biome-ignore lint/suspicious/noExplicitAny: the preview router has no typed route tree */}
            <RouterContextProvider router={previewRouter as any}>
                <ToastProvider>
                    <AnchoredToastProvider>
                        <CommandProvider>{children}</CommandProvider>
                    </AnchoredToastProvider>
                </ToastProvider>
            </RouterContextProvider>
        </QueryClientProvider>
    );
}
