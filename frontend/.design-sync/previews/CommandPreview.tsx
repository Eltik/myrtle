import { CommandPreview } from "frontend";

// The static ⌘K mock that sits in the hero's tilted glass panel: a fake search
// row, three operator results and two tool results. It has no state of its own -
// every surface is a click target that opens the real command palette.
//
// `.cmdPopup` applies `rotateX/rotateY` with fallbacks, so it only looks right
// when an ancestor supplies the perspective the hero panel sets (`1400px`).

const noop = () => {};

export const InHeroPanel = () => (
    <div className="mx-auto max-w-md" style={{ perspective: "1400px" }}>
        <CommandPreview onOpenCommand={noop} />
    </div>
);

export const FlatOnPage = () => (
    <div className="mx-auto max-w-md" style={{ perspective: "1400px", ["--tilt-x" as never]: "0deg", ["--tilt-y" as never]: "0deg" }}>
        <CommandPreview onOpenCommand={noop} />
    </div>
);

export const BesideCopy = () => (
    <div className="grid grid-cols-1 items-center gap-8 md:grid-cols-2">
        <div>
            <h2 className="m-0 mb-3 font-bold font-sans text-3xl text-foreground leading-tight tracking-tight">Jump anywhere.</h2>
            <p className="m-0 font-sans text-[15px] text-muted-foreground leading-relaxed">Operators, stages, enemies and every calculator are one keystroke away. The panel below is the resting preview the landing page shows before the palette opens.</p>
        </div>
        <div style={{ perspective: "1400px" }}>
            <CommandPreview onOpenCommand={noop} />
        </div>
    </div>
);
