import { HomeHero } from "frontend";

// The landing hero: version pill, headline, ⌘K hint, the two calls to action and
// the three live counters, with the tilted command-palette panel on the right.
//
// The counters come from `statsQueryOptions()`; the preview query client never
// retries and the server function is stubbed, so they resolve to the component's
// own "-" placeholder. That is the honest un-hydrated card.

const noop = () => {};

// One story only: the hero has no prop-driven variant axis. Its two branches are
// the counters (query-hydrated) and the second CTA, which swaps "Link Yostar
// account" for "View profile" when `useAuth()` returns a user — neither is
// reachable from a preview, and the signed-in CTA is covered by AuthDialog's own
// card. A second cell would be a pixel copy of this one.
export const Default = () => <HomeHero onOpenCommand={noop} />;
