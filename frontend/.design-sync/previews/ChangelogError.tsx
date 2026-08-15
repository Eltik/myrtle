import { ChangelogError } from "frontend";

// The changelog route's error boundary. `error` is whatever the loader threw,
// so the only axis worth sweeping is the shape of that value: an Error gets its
// message echoed in the mono callout, anything else falls back to the generic
// line.
export const RateLimited = () => <ChangelogError error={new Error("GitHub API responded 403: API rate limit exceeded for 203.0.113.44 (60 requests/hour without a token)")} />;

export const NetworkFailure = () => <ChangelogError error={new Error("fetch failed: getaddrinfo ENOTFOUND api.github.com")} />;

export const RepositoryMisconfigured = () => <ChangelogError error={new Error('GITHUB_REPO "Eltik/mrytle" could not be resolved — 404 Not Found')} />;

export const NonErrorThrown = () => <ChangelogError error={{ status: 502 }} />;
