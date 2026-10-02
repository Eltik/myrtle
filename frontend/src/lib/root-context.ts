import type { getSessionFn } from "#/lib/auth/server";
import type { IBootstrap } from "#/lib/i18n";
import { parseLocaleFromPath } from "#/lib/i18n/locale";

type Session = Awaited<ReturnType<typeof getSessionFn>>;

/**
 * Client-only memo of the two server fns the root `beforeLoad` awaits. Router-core re-runs
 * that `beforeLoad` on every client navigation and every hover preload, and each run was two
 * round trips for values that only change when this client changes them or reloads.
 *
 * Every client path that changes the session updates the memo: login and `finishLogin` prime
 * it with the session their response carries and logout primes it with null, each before its
 * `router.invalidate()`, and a failed login, a settings save, a roster resync and an account
 * disconnect drop it with `forgetSession()`. A change this client did not make (a sign-out in another
 * tab, an expiry server side) shows within SESSION_MAX_AGE_MS, where it used to show on the
 * next navigation.
 *
 * The bootstrap is keyed on the path locale, but the locale only changes through a full page
 * load (`useLocaleSwitch`), which resets this module anyway. A catalog republished mid-session
 * now shows on the next page load rather than the next navigation.
 *
 * SESSION_MAX_AGE_MS = 0 and MEMO_BOOTSTRAP = false restore a fetch on every run.
 */
const SESSION_MAX_AGE_MS = 60_000;
const MEMO_BOOTSTRAP = true;

const isClient = typeof window !== "undefined";

// Promises rather than values, so concurrent preloads share one request and a fetch that
// started before `forgetSession()` cannot write its stale answer back afterwards.
let session: { promise: Promise<Session>; at: number } | null = null;
let bootstrap: { promise: Promise<IBootstrap>; locale: string | null } | null = null;

function pathLocale(): string | null {
    return parseLocaleFromPath(window.location.pathname).locale;
}

export function memoSession(fetchSession: () => Promise<Session>): Promise<Session> {
    if (!isClient || SESSION_MAX_AGE_MS <= 0) return fetchSession();
    const now = Date.now();
    if (session && now - session.at < SESSION_MAX_AGE_MS) return session.promise;
    const entry = { promise: fetchSession(), at: now };
    session = entry;
    entry.promise.catch(() => {
        if (session === entry) session = null;
    });
    return entry.promise;
}

export function memoBootstrap(fetchBootstrap: () => Promise<IBootstrap>): Promise<IBootstrap> {
    if (!isClient || !MEMO_BOOTSTRAP) return fetchBootstrap();
    const locale = pathLocale();
    if (bootstrap && bootstrap.locale === locale) return bootstrap.promise;
    const entry = { promise: fetchBootstrap(), locale };
    bootstrap = entry;
    // A redirecting bootstrap is never reused: the next run must ask again.
    entry.promise.then(
        (b) => {
            if (b.redirectTo && bootstrap === entry) bootstrap = null;
        },
        () => {
            if (bootstrap === entry) bootstrap = null;
        },
    );
    return entry.promise;
}

/** Seed from the hydrated root context, so the first client navigation does not refetch
 *  what the server render already resolved. Never overwrites a newer entry. */
export function seedRootContext(user: Session, i18n: IBootstrap | undefined): void {
    if (!isClient) return;
    if (!session && SESSION_MAX_AGE_MS > 0) session = { promise: Promise.resolve(user), at: Date.now() };
    if (!bootstrap && MEMO_BOOTSTRAP && i18n && !i18n.redirectTo) bootstrap = { promise: Promise.resolve(i18n), locale: pathLocale() };
}

export function primeSession(user: Session): void {
    if (isClient && SESSION_MAX_AGE_MS > 0) session = { promise: Promise.resolve(user), at: Date.now() };
}

export function forgetSession(): void {
    session = null;
}
