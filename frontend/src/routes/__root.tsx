import type { QueryClient } from "@tanstack/react-query";
import { createRootRouteWithContext, HeadContent, Outlet, redirect, Scripts, useNavigate, useRouter, useRouterState } from "@tanstack/react-router";
import { useSelector } from "@tanstack/react-store";
import { lazy, Suspense, useEffect, useRef } from "react";
import { ReleaseNoteDialog } from "#/components/changelog/ReleaseNoteDialog";
import { useMusicLive } from "#/components/gallery/music/player/presence";
import { LazyAuthDialog } from "#/components/header/impl/LazyAuthDialog";
import { AnchoredToastProvider, ToastProvider } from "#/components/ui/toast";
import { useHydrated } from "#/hooks/use-hydrated";
import { isSignInPrompt } from "#/lib/auth/search";
import { getSessionFn } from "#/lib/auth/server";
import { authActions, authStore } from "#/lib/auth/store";
import { CommandProvider } from "#/lib/command-context";
import type { IBootstrap } from "#/lib/i18n";
import { DEFAULT_LOCALE, directionForLocale, I18nProvider } from "#/lib/i18n";
import { getI18nBootstrapFn } from "#/lib/i18n/server";
import { legacyRedirect } from "#/lib/legacy-routes";
import { metaT } from "#/lib/meta";
import { memoBootstrap, memoSession, seedRootContext } from "#/lib/root-context";
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

const MusicHost = lazy(() => import("#/components/gallery/music/player/MusicPlayerDock").then((m) => ({ default: m.MusicHost })));

const TanStackDevtoolsRoot = import.meta.env.DEV ? lazy(() => import("../integrations/tanstack-query/devtools")) : null;

const THEME_INIT_SCRIPT = `(function(){try{var stored=window.localStorage.getItem('theme');var mode=(stored==='light'||stored==='dark'||stored==='auto')?stored:'auto';var prefersDark=window.matchMedia('(prefers-color-scheme: dark)').matches;var resolved=mode==='auto'?(prefersDark?'dark':'light'):mode;var root=document.documentElement;root.classList.remove('light','dark');root.classList.add(resolved);root.style.colorScheme=resolved;
var raw=window.localStorage.getItem('myrtle-accent');
if(raw){var setVars=function(p,fg,gA,gB,gC){root.style.setProperty('--primary',p);root.style.setProperty('--primary-foreground',fg);root.style.setProperty('--ring',p);root.style.setProperty('--chart-1',p);root.style.setProperty('--sidebar-primary',p);root.style.setProperty('--sidebar-primary-foreground',fg);root.style.setProperty('--sidebar-ring',p);root.style.setProperty('--glow-primary',gA);root.style.setProperty('--glow-primary-intense',gB);root.style.setProperty('--glow-text-icon',gC);};
if(raw.indexOf('h:')===0){var h=parseFloat(raw.slice(2));if(!isNaN(h)){var L=resolved==='dark'?'0.75':'0.58';var C=resolved==='dark'?'0.15':'0.22';var FG=resolved==='dark'?'oklch(0.13 0.005 285)':'oklch(0.985 0.002 285)';setVars('oklch('+L+' '+C+' '+h+')',FG,'oklch('+L+' '+C+' '+h+' / '+(resolved==='dark'?'0.5':'0.35')+')','oklch('+L+' '+C+' '+h+' / '+(resolved==='dark'?'0.8':'0.55')+')','oklch('+L+' '+C+' '+h+' / '+(resolved==='dark'?'0.6':'0.55')+')');}}
else if(raw.indexOf('c:')===0){var hex=raw.slice(2);if(/^#[0-9a-fA-F]{6}$/.test(hex)){var r=parseInt(hex.slice(1,3),16)/255,g=parseInt(hex.slice(3,5),16)/255,b=parseInt(hex.slice(5,7),16)/255;var lin=function(c){return c<=0.04045?c/12.92:Math.pow((c+0.055)/1.055,2.4)};var lum=0.2126*lin(r)+0.7152*lin(g)+0.0722*lin(b);var FG=lum>0.55?'oklch(0.13 0.005 285)':'oklch(0.985 0.002 285)';var R=Math.round(r*255),G=Math.round(g*255),B=Math.round(b*255);setVars(hex,FG,'rgb('+R+' '+G+' '+B+' / 0.5)','rgb('+R+' '+G+' '+B+' / 0.8)','rgb('+R+' '+G+' '+B+' / 0.6)');}}
}}catch(e){}})();`;

export const Route = createRootRouteWithContext<IMyRouterContext>()({
    beforeLoad: async ({ location }): Promise<{ user: Awaited<ReturnType<typeof getSessionFn>> } & IRootContext> => {
        // A URL an earlier version of the site served, before the session and catalog
        // round trip it does not need. 301 so search engines move the page's rank too.
        const legacy = legacyRedirect(location.pathname, location.search as Record<string, unknown>);
        if (legacy) throw redirect({ href: legacy, statusCode: 301 });

        // One round trip, not two: the catalog is needed for the very first
        // painted character, so it must not queue behind the session. Both are
        // memoized on the client, where this runs on every navigation and every
        // hover preload; see `lib/root-context.ts` for what drops them.
        const [user, i18n] = await Promise.all([memoSession(() => getSessionFn()), memoBootstrap(() => getI18nBootstrapFn())]);

        // The URL claimed a locale the backend does not serve. `href` rather
        // than `to`, because `to` would be rebuilt through the router's
        // locale rewrite and put the bad prefix straight back on.
        if (i18n.redirectTo) throw redirect({ href: i18n.redirectTo });

        return { user, i18n };
    },
    head: ({ match }) => {
        // No `useT()` in `head()`; the context already has the bootstrap. See `lib/meta.ts`.
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
    const { user, i18n } = Route.useRouteContext();

    useEffect(() => {
        authActions.setUserFromRoute(user ?? null);
    }, [user]);

    useEffect(() => {
        seedRootContext(user ?? null, i18n);
    }, [user, i18n]);

    return (
        <>
            <Outlet />
            <GlobalAuthDialog />
            <ReleaseNoteDialog />
            <SiteMusic />
        </>
    );
}

/**
 * The music player, one instance for the whole site. It mounts on the music
 * page, and on every other route once the player has sounded in this page
 * load (`presence.ts` has no engine in it), so a visitor who never plays music
 * never downloads the player. The dynchar harness is scored pixel for pixel,
 * so it never gets the bar.
 */
function SiteMusic() {
    const live = useMusicLive();
    const hydrated = useHydrated();
    const pathname = useRouterState({ select: (s) => s.location.pathname });
    const onMusicPage = pathname.startsWith("/gallery/music");
    // Client only: the player's store is the browser's (the music page fills
    // it from localStorage), and `@tanstack/react-store` hands hydration the
    // live store as its server snapshot, so a server-rendered dock mismatched
    // the HTML. The server never had a track, so it drew no bar to keep.
    if (!hydrated || (!live && !onMusicPage) || pathname.startsWith("/dyntest")) return null;
    return (
        <Suspense fallback={null}>
            <MusicHost offPage={!onMusicPage} />
        </Suspense>
    );
}

function GlobalAuthDialog() {
    const dialogOpen = useSelector(authStore, (s) => s.dialogOpen);
    const user = useSelector(authStore, (s) => s.user);
    // The store learns the session from `RootComponent`'s effect, which runs
    // AFTER this child's effects on the first render, so on a hydrating page
    // the store still says signed out. The route context already knows.
    const routeUser = Route.useRouteContext({ select: (c) => c.user });
    const navigate = useNavigate();
    const router = useRouter();
    const search = useRouterState({ select: (s) => s.location.search as Record<string, unknown> });
    const pathname = useRouterState({ select: (s) => s.location.pathname });
    const promptedRef = useRef<string | null>(null);
    const lastUserRef = useRef(user);

    useEffect(() => {
        // `?auth=1` parses as the number 1, the app's older redirects sent the
        // string "1" (`?auth=%221%22`): `isSignInPrompt` takes every spelling.
        if (!isSignInPrompt(search.auth)) {
            promptedRef.current = null;
            return;
        }
        const key = `${pathname}?auth=1`;
        if (promptedRef.current === key) return;
        promptedRef.current = key;

        const nextParam = typeof search.next === "string" ? search.next : null;
        if (!user && !routeUser) {
            authActions.openLoginDialog(nextParam);
        }

        const cleaned: Record<string, unknown> = { ...search };
        delete cleaned.auth;
        delete cleaned.next;
        navigate({ to: pathname, search: cleaned, replace: true });
    }, [search, pathname, user, routeUser, navigate]);

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

    return <LazyAuthDialog open={dialogOpen} onOpenChange={authActions.setDialogOpen} />;
}

function SiteChrome({ children }: { children: React.ReactNode }) {
    const pathname = useRouterState({ select: (s) => s.location.pathname });
    const isAdmin = pathname.startsWith("/admin");

    // The admin panel sits under the site header (its own section bar renders
    // below it) but has no footer: it is a working surface, not a page to scroll past.
    if (isAdmin) {
        return (
            <>
                <Header />
                <ToastProvider>
                    <AnchoredToastProvider>{children}</AnchoredToastProvider>
                </ToastProvider>
            </>
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
    // `/dyntest` is the dynchar parity harness, scored pixel-for-pixel against in-game captures.
    // The devtools launcher paints a 56×56 badge bottom-right that CSS can't hide
    // (`visibility: hidden !important` everywhere, a second pass after settle, and hiding
    // whatever `elementsFromPoint` reports all left the frame byte-identical). It hit scored
    // frames (Virtuosa t=2, Mlynar t=4, Skadi t=3): +0.34-0.79 MAD per beat, +0.043-0.113 MADC
    // per skin mean. So it isn't mounted on that route.
    const devtoolsPath = useRouterState({ select: (s) => s.location.pathname });
    const showDevtools = TanStackDevtoolsRoot !== null && !devtoolsPath.startsWith("/dyntest");

    // `lang` drives hyphenation, the CJK font fallback chain, `:lang()` rules and screen reader
    // pronunciation. The provider lives in the shell, not `RootComponent`, because `SiteChrome`
    // renders here, above it. One level lower, Header and Footer read the empty default context:
    // English text and language switchers with `available: []`.
    const i18n = Route.useRouteContext({ select: (c) => c.i18n });
    const locale = i18n?.locale ?? DEFAULT_LOCALE;
    const alternates = i18n?.available.map((l) => l.code) ?? [];
    const pathname = useRouterState({ select: (s) => s.location.pathname });

    // Canonical, og:url and hreflang for every route, here because a route's `head()` doesn't
    // know the locale, and one missed call site would make the English page a translation's
    // canonical. `pathname` already has the locale prefix stripped by the rewrite.
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
                <I18nProvider locale={locale} available={i18n?.available ?? []} messages={i18n?.messages ?? {}} gamedataServer={i18n?.gamedataServer} localeGamedataServer={i18n?.localeGamedataServer} gamedataServerPicked={i18n?.gamedataServerPicked} gamedataServers={i18n?.gamedataServers}>
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
