// Design-sync stub for `#/hooks/use-auth`.
//
// The real hook opens with `useRouteContext({ from: "__root__" })`, which throws
//   Invariant failed: Could not find an active match from "__root__"
// under the preview provider: it mounts the router via `RouterContextProvider`
// over an unloaded route tree, so `router.state.matches` is empty. The throw
// happens before any JSX, and capture reports zero errors — the card is simply
// blank. With 21 call sites across src/, that silently blanked cards in several
// feature groups (admin shell + screens, gacha pages, settings).
//
// It keeps the real hook's return shape but reads the auth store directly, which
// is a plain TanStack Store singleton and needs no router. Previews therefore
// render the signed-out branch, which is the honest state for a design-system
// card: there is no session in a preview, and the login/logout server functions
// are stubbed out anyway.
//
// Wired in through `compilerOptions.paths` in .design-sync/tsconfig.ds.json.

import { useStore } from "@tanstack/react-store";
import { authStore } from "#/lib/auth/store";

export function useAuth() {
    const user = useStore(authStore, (s) => s.user);
    const loading = useStore(authStore, (s) => s.status === "loading");

    return {
        user,
        loading,
        isAuthenticated: user !== null,
        login: async () => {
            throw new Error("design-sync: authentication is not available in a preview");
        },
        logout: async () => {},
        fetchUser: async () => null,
    };
}
