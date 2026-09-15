import type { QueryClient } from "@tanstack/react-query";
import { createRootRouteWithContext, HeadContent, Outlet, redirect, Scripts, useNavigate, useRouter, useRouterState } from "@tanstack/react-router";
import { useSelector } from "@tanstack/react-store";
import { lazy, Suspense, useEffect, useRef } from "react";
import { ReleaseNoteDialog } from "#/components/changelog/ReleaseNoteDialog";
import { AuthDialog } from "#/components/header/impl/AuthDialog";
import { AnchoredToastProvider, ToastProvider } from "#/components/ui/toast";
import { getSessionFn } from "#/lib/auth/server";
import { authActions, authStore } from "#/lib/auth/store";
import { CommandProvider } from "#/lib/command-context";
import type { IBootstrap } from "#/lib/i18n";
import { DEFAULT_LOCALE, directionForLocale, I18nProvider } from "#/lib/i18n";
import { getI18nBootstrapFn } from "#/lib/i18n/server";
import { metaT } from "#/lib/meta";
import { absolute, localizedPath, seo } from "#/lib/seo";
import Footer from "../components/Footer";
import Header from "../components/header/Header";
import { RouterProgress } from "../components/RouterProgress";
import appCss from "../styles.css?url";

interface IMyRouterContext {
    queryClient: QueryClient;
}

/** What `beforeLoad` adds to the context, alongside `user`. */
interface IRootContext {
    i18n: IBootstrap;
}

const TanStackDevtoolsRoot = import.meta.env.DEV ? lazy(() => import("../integrations/tanstack-query/devtools")) : null;

const THEME_INIT_SCRIPT = `(function(){try{var stored=window.localStorage.getItem('theme');var mode=(stored==='light'||stored==='dark'||stored==='auto')?stored:'auto';var prefersDark=window.matchMedia('(prefers-color-scheme: dark)').matches;var resolved=mode==='auto'?(prefersDark?'dark':'light'):mode;var root=document.documentElement;root.classList.remove('light','dark');root.classList.add(resolved);root.style.colorScheme=resolved;
var raw=window.localStorage.getItem('myrtle-accent');
if(raw){var setVars=function(p,fg,gA,gB,gC){root.style.setProperty('--primary',p);root.style.setProperty('--primary-foreground',fg);root.style.setProperty('--ring',p);root.style.setProperty('--chart-1',p);root.style.setProperty('--sidebar-primary',p);root.style.setProperty('--sidebar-primary-foreground',fg);root.style.setProperty('--sidebar-ring',p);root.style.setProperty('--glow-primary',gA);root.style.setProperty('--glow-primary-intense',gB);root.style.setProperty('--glow-text-icon',gC);};
if(raw.indexOf('h:')===0){var h=parseFloat(raw.slice(2));if(!isNaN(h)){var L=resolved==='dark'?'0.75':'0.58';var C=resolved==='dark'?'0.15':'0.22';var FG=resolved==='dark'?'oklch(0.13 0.005 285)':'oklch(0.985 0.002 285)';setVars('oklch('+L+' '+C+' '+h+')',FG,'oklch('+L+' '+C+' '+h+' / '+(resolved==='dark'?'0.5':'0.35')+')','oklch('+L+' '+C+' '+h+' / '+(resolved==='dark'?'0.8':'0.55')+')','oklch('+L+' '+C+' '+h+' / '+(resolved==='dark'?'0.6':'0.55')+')');}}
else if(raw.indexOf('c:')===0){var hex=raw.slice(2);if(/^#[0-9a-fA-F]{6}$/.test(hex)){var r=parseInt(hex.slice(1,3),16)/255,g=parseInt(hex.slice(3,5),16)/255,b=parseInt(hex.slice(5,7),16)/255;var lin=function(c){return c<=0.04045?c/12.92:Math.pow((c+0.055)/1.055,2.4)};var lum=0.2126*lin(r)+0.7152*lin(g)+0.0722*lin(b);var FG=lum>0.55?'oklch(0.13 0.005 285)':'oklch(0.985 0.002 285)';var R=Math.round(r*255),G=Math.round(g*255),B=Math.round(b*255);setVars(hex,FG,'rgb('+R+' '+G+' '+B+' / 0.5)','rgb('+R+' '+G+' '+B+' / 0.8)','rgb('+R+' '+G+' '+B+' / 0.6)');}}
}}catch(e){}})();`;

export const Route = createRootRouteWithContext<IMyRouterContext>()({
    beforeLoad: async (): Promise<{ user: Awaited<ReturnType<typeof getSessionFn>> } & IRootContext> => {
        // One round trip, not two: the catalog is needed for the very first
        // painted character, so it must not queue behind the session.
        const [user, i18n] = await Promise.all([getSessionFn(), getI18nBootstrapFn()]);

        // The URL claimed a locale the backend does not serve. `href` rather
        // than `to`, because `to` would be rebuilt through the router's
        // locale rewrite and put the bad prefix straight back on.
        if (i18n.redirectTo) throw redirect({ href: i18n.redirectTo });

        return { user, i18n };
    },
    head: ({ match }) => {
        // `head()` is not a component, so there is no `useT()` here - but the
        // match's context is the resolved route context, which already carries
        // the `i18n` bootstrap `beforeLoad` above loaded. See `lib/meta.ts`.
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("root.title"),
            description: t("root.description"),
            locale: match.context.i18n?.locale,
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

    return (
        <>
            <Outlet />
            <GlobalAuthDialog />
            <ReleaseNoteDialog />
        </>
    );
}

function GlobalAuthDialog() {
    const dialogOpen = useSelector(authStore, (s) => s.dialogOpen);
    const user = useSelector(authStore, (s) => s.user);
    const navigate = useNavigate();
    const router = useRouter();
    const search = useRouterState({ select: (s) => s.location.search as Record<string, unknown> });
    const pathname = useRouterState({ select: (s) => s.location.pathname });
    const promptedRef = useRef<string | null>(null);
    const lastUserRef = useRef(user);

    useEffect(() => {
        const authParam = search.auth;
        if (authParam !== "1") {
            promptedRef.current = null;
            return;
        }
        const key = `${pathname}?auth=1`;
        if (promptedRef.current === key) return;
        promptedRef.current = key;

        const nextParam = typeof search.next === "string" ? search.next : null;
        if (!user) {
            authActions.openLoginDialog(nextParam);
        }

        const cleaned: Record<string, unknown> = { ...search };
        delete cleaned.auth;
        delete cleaned.next;
        navigate({ to: pathname, search: cleaned, replace: true });
    }, [search, pathname, user, navigate]);

    useEffect(() => {
        const previous = lastUserRef.current;
        lastUserRef.current = user;
        if (previous || !user) return;
        const target = authActions.consumePostLoginRedirect();
        if (target) {
            try {
                const url = new URL(target, window.location.origin);
                router.history.push(`${url.pathname}${url.search}${url.hash}`);
            } catch {
                router.history.push(target);
            }
        }
    }, [user, router]);

    return <AuthDialog open={dialogOpen} onOpenChange={authActions.setDialogOpen} />;
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
    // `/dyntest` is the dynchar parity MEASUREMENT harness - its canvas is scored
    // pixel-for-pixel against in-game captures. The devtools launcher paints a 56×56
    // badge over the bottom-right of that canvas, and hiding it from the DOM does not
    // work: `visibility: hidden !important` over every element, a second pass after a
    // settle, and hiding whatever `elementsFromPoint` reports painting there all left
    // the frame byte-identical, with the button still computing `visibility: visible`.
    // It therefore reached scored frames (Virtuosa t=2, Mlynar t=4, Skadi t=3), lifting
    // the affected beat by 0.34–0.79 MAD and each skin's mean by 0.043–0.113 MADC. Not
    // mounting it on that one route is the only reliable fix and costs nothing - the
    // page has no UI to inspect, and every other route keeps its devtools.
    const devtoolsPath = useRouterState({ select: (s) => s.location.pathname });
    const showDevtools = TanStackDevtoolsRoot !== null && !devtoolsPath.startsWith("/dyntest");

    // Getting `lang` right is not cosmetic: it drives hyphenation, the font
    // fallback chain for CJK, `:lang()` rules, and how a screen reader
    // pronounces the page.
    // The whole i18n bootstrap, not just the locale: the provider has to live
    // HERE rather than in `RootComponent`, because `SiteChrome` - the Header
    // and Footer - is rendered by this shell, which sits ABOVE
    // `RootComponent`. With the provider one level lower, every string in the
    // header and footer read the default empty context and fell back to
    // English whatever locale the page was, and both language switchers saw
    // `available: []` and rendered nothing at all.
    const i18n = Route.useRouteContext({ select: (c) => c.i18n });
    const locale = i18n?.locale ?? DEFAULT_LOCALE;
    const alternates = i18n?.available.map((l) => l.code) ?? [];
    const pathname = useRouterState({ select: (s) => s.location.pathname });

    // One canonical, one og:url and the whole hreflang set for every route.
    //
    // This lives here rather than in each route's `seo()` because only the
    // shell knows the locale - a route's `head()` has no access to it - and a
    // single missed call site would declare the English page as a
    // translation's canonical, which is how a translated site fails to get
    // indexed at all. `pathname` is the router's internal path, with the
    // locale prefix already stripped by the location rewrite, so it composes
    // cleanly with every locale.
    const canonical = absolute(localizedPath(pathname, locale));

    return (
        <html lang={locale} dir={directionForLocale(locale)} suppressHydrationWarning>
            <head>
                {/* biome-ignore lint/security/noDangerouslySetInnerHtml: theme init script */}
                <script dangerouslySetInnerHTML={{ __html: THEME_INIT_SCRIPT }} />
                <HeadContent />
                <link rel="canonical" href={canonical} />
                <meta property="og:url" content={canonical} />
                {alternates.length > 1
                    ? [
                          ...alternates.map((code) => <link key={code} rel="alternate" hrefLang={code} href={absolute(localizedPath(pathname, code))} />),
                          // `x-default` is what a crawler serves a visitor whose
                          // language matches none of ours.
                          <link key="x-default" rel="alternate" hrefLang="x-default" href={absolute(localizedPath(pathname, DEFAULT_LOCALE))} />,
                      ]
                    : null}
            </head>
            <body className="wrap-anywhere font-sans antialiased selection:bg-primary/30 selection:text-foreground">
                <RouterProgress />
                <I18nProvider locale={locale} available={i18n?.available ?? []} messages={i18n?.messages ?? {}} gamedataServer={i18n?.gamedataServer}>
                    <CommandProvider>
                        <SiteChrome>{children}</SiteChrome>
                    </CommandProvider>
                </I18nProvider>
                {showDevtools && TanStackDevtoolsRoot ? (
                    <Suspense fallback={null}>
                        <TanStackDevtoolsRoot />
                    </Suspense>
                ) : null}
                <Scripts />
            </body>
        </html>
    );
}
