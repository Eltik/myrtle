import { SectionTitle } from "frontend";

// A release-planner section heading with an optional row count.
export const WithCount = () => (
    <div className="w-full max-w-md p-4">
        <SectionTitle count={12}>Upcoming events</SectionTitle>
    </div>
);

export const WithoutCount = () => (
    <div className="w-full max-w-md p-4">
        <SectionTitle>Headhunting banners</SectionTitle>
    </div>
);

// Stacked above the content it heads, as the tabs use it.
export const AboveList = () => (
    <div className="flex w-full max-w-md flex-col p-4">
        <SectionTitle count={3}>Arriving soon</SectionTitle>
        <ul className="m-0 flex list-none flex-col gap-1 p-0 font-sans text-[13px] text-muted-foreground">
            <li>Adventure That Cannot Wait for the Sun - Rerun</li>
            <li>Critical Phase Transition</li>
            <li>Cantilena Puppae</li>
        </ul>
    </div>
);
