import { FeatureStrip } from "frontend";

// The three accented value-prop cards directly under the landing hero. The
// content is baked into the component (live data / roster sync / tools), so the
// stories vary the page context rather than props: it takes none.
//
// The strip lays itself out as `w-[min(1080px,calc(100%-2rem))]`, one column
// below `md` and three above it.

export const Default = () => <FeatureStrip />;

export const InPageSection = () => (
    <div className="w-full py-4">
        <div className="mx-auto mb-6 w-[min(1080px,calc(100%-2rem))]">
            <p className="m-0 mb-2 font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-widest">Why myrtle</p>
            <h2 className="m-0 mb-1.5 font-bold font-sans text-3xl text-foreground leading-tight tracking-tight">Everything a Doctor keeps a tab open for.</h2>
            <p className="m-0 max-w-[60ch] font-sans text-[14.5px] text-muted-foreground leading-relaxed">The strip below is how the landing page hands off from the hero into the tier-list gallery.</p>
        </div>
        <FeatureStrip />
    </div>
);
