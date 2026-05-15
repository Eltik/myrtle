import { TanStackDevtools } from "@tanstack/react-devtools";
import type { QueryClient } from "@tanstack/react-query";
import { createRootRouteWithContext, HeadContent, Outlet, Scripts, useRouterState } from "@tanstack/react-router";
import { TanStackRouterDevtoolsPanel } from "@tanstack/react-router-devtools";
import { useEffect } from "react";
import { AnchoredToastProvider, ToastProvider } from "#/components/ui/toast";
import { getSessionFn } from "#/lib/auth/server";
import { authActions } from "#/lib/auth/store";
import { CommandProvider } from "#/lib/command-context";
import { seo } from "#/lib/seo";
import Footer from "../components/Footer";
import Header from "../components/header/Header";
import { RouterProgress } from "../components/RouterProgress";
import TanStackQueryDevtools from "../integrations/tanstack-query/devtools";
import appCss from "../styles.css?url";

interface IMyRouterContext {
    queryClient: QueryClient;
}

const THEME_INIT_SCRIPT = `(function(){try{var stored=window.localStorage.getItem('theme');var mode=(stored==='light'||stored==='dark'||stored==='auto')?stored:'auto';var prefersDark=window.matchMedia('(prefers-color-scheme: dark)').matches;var resolved=mode==='auto'?(prefersDark?'dark':'light'):mode;var root=document.documentElement;root.classList.remove('light','dark');root.classList.add(resolved);if(mode==='auto'){root.removeAttribute('data-theme')}else{root.setAttribute('data-theme',mode)}root.style.colorScheme=resolved;
var raw=window.localStorage.getItem('myrtle-accent');
if(raw){var setVars=function(p,fg,gA,gB,gC){root.style.setProperty('--primary',p);root.style.setProperty('--primary-foreground',fg);root.style.setProperty('--ring',p);root.style.setProperty('--chart-1',p);root.style.setProperty('--sidebar-primary',p);root.style.setProperty('--sidebar-primary-foreground',fg);root.style.setProperty('--sidebar-ring',p);root.style.setProperty('--glow-primary',gA);root.style.setProperty('--glow-primary-intense',gB);root.style.setProperty('--glow-text-icon',gC);};
if(raw.indexOf('h:')===0){var h=parseFloat(raw.slice(2));if(!isNaN(h)){var L=resolved==='dark'?'0.75':'0.58';var C=resolved==='dark'?'0.15':'0.22';var FG=resolved==='dark'?'oklch(0.13 0.005 285)':'oklch(0.985 0.002 285)';setVars('oklch('+L+' '+C+' '+h+')',FG,'oklch('+L+' '+C+' '+h+' / '+(resolved==='dark'?'0.5':'0.35')+')','oklch('+L+' '+C+' '+h+' / '+(resolved==='dark'?'0.8':'0.55')+')','oklch('+L+' '+C+' '+h+' / '+(resolved==='dark'?'0.6':'0.55')+')');}}
else if(raw.indexOf('c:')===0){var hex=raw.slice(2);if(/^#[0-9a-fA-F]{6}$/.test(hex)){var r=parseInt(hex.slice(1,3),16)/255,g=parseInt(hex.slice(3,5),16)/255,b=parseInt(hex.slice(5,7),16)/255;var lin=function(c){return c<=0.04045?c/12.92:Math.pow((c+0.055)/1.055,2.4)};var lum=0.2126*lin(r)+0.7152*lin(g)+0.0722*lin(b);var FG=lum>0.55?'oklch(0.13 0.005 285)':'oklch(0.985 0.002 285)';var R=Math.round(r*255),G=Math.round(g*255),B=Math.round(b*255);setVars(hex,FG,'rgb('+R+' '+G+' '+B+' / 0.5)','rgb('+R+' '+G+' '+B+' / 0.8)','rgb('+R+' '+G+' '+B+' / 0.6)');}}
}}catch(e){}})();`;

export const Route = createRootRouteWithContext<IMyRouterContext>()({
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

function SiteChrome({ children }: { children: React.ReactNode }) {
    const pathname = useRouterState({ select: (s) => s.location.pathname });
    const isAdmin = pathname.startsWith("/admin");

    if (isAdmin) {
        return (
            <ToastProvider>
                <AnchoredToastProvider>{children}</AnchoredToastProvider>
            </ToastProvider>
        );
    }

    return (
        <>
            <Header />
            <ToastProvider>
                <AnchoredToastProvider>{children}</AnchoredToastProvider>
            </ToastProvider>
            <Footer />
        </>
    );
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
                <RouterProgress />
                <CommandProvider>
                    <SiteChrome>{children}</SiteChrome>
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
                        TanStackQueryDevtools,
                    ]}
                />
                <Scripts />
            </body>
        </html>
    );
}
