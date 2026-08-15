import { ChangelogPage } from "frontend";

// The /changelog route container. It pulls commits from GitHub through a server
// function the design bundle stubs, so the hero, the repo chip, the summary
// tiles and the range tabs render for real while the timeline settles on its
// empty state — the widest window with no commits in it.
export const CommitsUnavailable = () => <ChangelogPage />;
