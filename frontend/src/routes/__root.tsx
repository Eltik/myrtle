import { TanStackDevtools } from "@tanstack/react-devtools";
import type { QueryClient } from "@tanstack/react-query";
import { createRootRouteWithContext, HeadContent, Outlet, Scripts } from "@tanstack/react-router";
import { TanStackRouterDevtoolsPanel } from "@tanstack/react-router-devtools";
import { useEffect } from "react";
import { AnchoredToastProvider, ToastProvider } from "#/components/ui/toast";
import { getSessionFn } from "#/lib/auth/server";
import { authActions } from "#/lib/auth/store";
import { CommandProvider } from "#/lib/command-context";
import { seo } from "#/lib/seo";
import Footer from "../components/Footer";
import Header from "../components/header/Header";
import TanStackQueryDevtools from "../integrations/tanstack-query/devtools";
import StoreDevtools from "../lib/demo-store-devtools";
import appCss from "../styles.css?url";

interface MyRouterContext {
    queryClient: QueryClient;
}

const THEME_INIT_SCRIPT = `(function(){try{var stored=window.localStorage.getItem('theme');var mode=(stored==='light'||stored==='dark'||stored==='auto')?stored:'auto';var prefersDark=window.matchMedia('(prefers-color-scheme: dark)').matches;var resolved=mode==='auto'?(prefersDark?'dark':'light'):mode;var root=document.documentElement;root.classList.remove('light','dark');root.classList.add(resolved);if(mode==='auto'){root.removeAttribute('data-theme')}else{root.setAttribute('data-theme',mode)}root.style.colorScheme=resolved;}catch(e){}})();`;

export const Route = createRootRouteWithContext<MyRouterContext>()({
    beforeLoad: async () => {
        const user = await getSessionFn();
        return { user };
    },
    head: () => {
        const { meta, links } = seo({
            title: "Myrtle",
            description: "Arknights companion - operators, rosters, tier lists.",
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links: [{ rel: "stylesheet", href: appCss }, ...links],
        };
    },
    component: RootComponent,
    shellComponent: RootDocument,
});

function RootComponent() {
    const { user } = Route.useRouteContext();

    useEffect(() => {
        authActions.setUser(user ?? null);
    }, [user]);

    return <Outlet />;
}

function RootDocument({ children }: { children: React.ReactNode }) {
    return (
        <html lang="en" suppressHydrationWarning>
            <head>
                {/* biome-ignore lint/security/noDangerouslySetInnerHtml: theme init script */}
                <script dangerouslySetInnerHTML={{ __html: THEME_INIT_SCRIPT }} />
                <HeadContent />
            </head>
            <body className="font-sans antialiased wrap-anywhere selection:bg-primary">
                <CommandProvider>
                    <Header />
                    <ToastProvider>
                        <AnchoredToastProvider>{children}</AnchoredToastProvider>
                    </ToastProvider>
                    <Footer />
                </CommandProvider>
                <TanStackDevtools
                    config={{
                        position: "bottom-right",
                    }}
                    plugins={[
                        {
                            name: "Tanstack Router",
                            render: <TanStackRouterDevtoolsPanel />,
                        },
                        StoreDevtools,
                        TanStackQueryDevtools,
                    ]}
                />
                <Scripts />
            </body>
        </html>
    );
}
