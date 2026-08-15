import { Health } from "frontend";

// The /health probe and the /stats snapshot are both stubbed server functions in
// a preview, so the screen renders its unreachable branch: dash-filled latency
// tiles with red status dots and the "endpoint unavailable" fallbacks.
export function ProbesUnreachable() {
    return <Health />;
}
