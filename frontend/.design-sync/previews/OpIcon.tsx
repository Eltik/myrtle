import { OpIcon } from "frontend";

// Originite Prime's HUD hexagon, sized to sit inline with text (16px default).
export const Inline = () => (
    <p className="m-0 p-4 font-sans text-[13px] text-foreground">
        Monthly card: 6 <OpIcon /> on purchase, plus 200 Orundum a day.
    </p>
);

// The sizes the planner uses: inline in an amount (14px), default (16px), and a
// larger one for a resource field label.
export const Sizes = () => (
    <div className="flex items-end gap-4 p-4">
        <OpIcon className="size-3.5" />
        <OpIcon />
        <OpIcon className="size-6" />
        <OpIcon className="size-8" />
    </div>
);
