/**
 * The `?auth=1` deep link that opens the sign-in dialog on any page.
 *
 * The router parses every search value as JSON first, so a hand-typed
 * `?auth=1` arrives as the NUMBER 1 and `?auth=true` as a boolean, while the
 * string "1" only survives as `?auth=%221%22`. Readers accept all three, and
 * the app's own redirects emit the number, which serializes as the clean
 * `?auth=1` that someone can type or share.
 */

/** The value the app writes; `defaultStringifySearch` renders it as `auth=1`. */
export const SIGN_IN_PROMPT = 1 as const;

/** True for every spelling of the sign-in prompt the router can hand over. */
export function isSignInPrompt(value: unknown): boolean {
    return value === SIGN_IN_PROMPT || value === "1" || value === true;
}

/** Search for a redirect to the sign-in prompt that returns to `next` afterwards. */
export function signInSearch(next: string): { auth: typeof SIGN_IN_PROMPT; next: string } {
    return { auth: SIGN_IN_PROMPT, next };
}
