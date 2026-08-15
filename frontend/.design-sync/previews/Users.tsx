import { Users } from "frontend";

// The admin panel's Users screen: search rail, per-server segmented tabs, a
// result count and the doctor table. `/search` is a stubbed server function in
// the design bundle, so the table resolves to its no-results state while the
// full toolbar chrome still renders.
export const DirectoryUnavailable = () => <Users />;
