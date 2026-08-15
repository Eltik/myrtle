import { Browse } from "frontend";

// `Browse` is the /tier-lists route container: it reads typed search params and
// pulls the whole catalogue through a server function. Both are stubbed in the
// design bundle, so the preview renders the page's real load-failure branch —
// hero, official rail, filter toolbar and the retry panel.
export const LoadFailed = () => <Browse />;
