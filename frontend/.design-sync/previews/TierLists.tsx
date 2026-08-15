import { TierLists } from "frontend";

// The "Tier lists, at a glance" gallery section on the landing page: heading,
// tag filter tabs and a three-up grid of `TierListCard`s, closed by the
// "Browse all tier lists" link.
//
// It reads `homeTierListsQueryOptions()` and takes no props, so a preview can
// only show a data-free branch. The server function is stubbed and the preview
// query client has `retry: false`, so the query rejects immediately and the
// section renders its own failure placeholder rather than spinning. The tag
// tabs stay hidden because they are derived from the (empty) result set.

export const LoadFailed = () => <TierLists />;
