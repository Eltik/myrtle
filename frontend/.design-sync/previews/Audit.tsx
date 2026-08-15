import { Audit } from "frontend";

// The append-only audit feed comes from a stubbed server function in a preview,
// so the screen renders its toolbar (search + field tabs + counts) above the
// load-failure branch. That is the whole statically reachable surface.
export function AuditFeedUnavailable() {
    return <Audit />;
}
