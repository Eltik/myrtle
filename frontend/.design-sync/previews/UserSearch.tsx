import { UserSearch } from "frontend";
import { useEffect } from "react";

/**
 * `UserSearch` is the whole `/user/search` route container: it reads its typed
 * search params off the route, debounces the nickname query and pages through
 * `searchUsersQueryOptions`. That server function is stubbed in previews, so the
 * two reachable states are its browse-mode and searching-mode empty states,
 * both under the page's real chrome.
 */
export const BrowseEmpty = () => <UserSearch />;

// The query is state, not a prop: it lives in `useState` behind a 350ms debounce.
// Typing into the real input (as a native input event, so React's onChange runs)
// is the only way to photograph the searching branch.
function TypeQuery({ q }: { q: string }) {
    useEffect(() => {
        let f2 = 0;
        const f1 = requestAnimationFrame(() => {
            f2 = requestAnimationFrame(() => {
                const input = document.querySelector<HTMLInputElement>('input[aria-label="Search doctors"]');
                const setValue = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value")?.set;
                if (input && setValue) {
                    setValue.call(input, q);
                    input.dispatchEvent(new Event("input", { bubbles: true }));
                    input.blur();
                }
            });
        });
        return () => {
            cancelAnimationFrame(f1);
            cancelAnimationFrame(f2);
        };
    }, [q]);
    return null;
}

/** Searching mode: the query echo, the clear affordance and the no-match empty. */
export const NoMatches = () => (
    <>
        <TypeQuery q="Ceylonade" />
        <UserSearch />
    </>
);
