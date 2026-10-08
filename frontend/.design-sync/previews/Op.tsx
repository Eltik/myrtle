import { Op } from "frontend";

// An Originite Prime amount: mono figures with the HUD icon after them.
export const Default = () => (
    <div className="p-4 text-foreground">
        <Op value={120} />
    </div>
);

// Signed, tinted the way the planner's income/expense rows use it.
export const Signed = () => (
    <div className="flex flex-col gap-1.5 p-4 text-[13px]">
        <Op value={30} sign="+" className="text-emerald-500" />
        <Op value={18} sign="-" className="text-rose-400" />
    </div>
);

// Inline in a sentence.
export const InSentence = () => (
    <p className="m-0 p-4 font-sans text-[13px] text-muted-foreground">
        Clearing every new stage of the event pays out <Op value={30} className="text-foreground" /> on first clear.
    </p>
);
