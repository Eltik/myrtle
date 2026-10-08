import { BannersTab } from "frontend";

// `BannersTab` is a data container: it loads the /release payloads through server
// functions, which reject in the design bundle with a product-shaped message. This
// renders the tab's real load-failure branch.
export const LoadFailed = () => (
    <div className="w-full p-4">
        <BannersTab today={new Date("2026-09-01T12:00:00+09:00")} />
    </div>
);
