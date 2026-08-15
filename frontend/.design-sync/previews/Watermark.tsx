import { Watermark } from "frontend";

/** The board surface MapView paints; the watermark is pinned to its bottom-left. */
const Surface = ({ children }: { children?: React.ReactNode }) => (
    <div className="relative h-40 w-full overflow-hidden rounded-[14px] border border-border bg-[#181818] bg-[linear-gradient(90deg,#0a0a0a_1.5px,transparent_1%),linear-gradient(#0a0a0a_1.5px,transparent_1%)] bg-position-[50%] bg-size-[2.5px_2.5px]">{children}</div>
);

export const StageCode = () => (
    <div className="max-w-2xl">
        <Surface>
            <Watermark text="4-10" />
        </Surface>
    </div>
);

export const SubStageCode = () => (
    <div className="max-w-2xl">
        <Surface>
            <Watermark text="S4-1" />
        </Surface>
    </div>
);

/** Codeless procedural nodes fall back to the literal "Map". */
export const Fallback = () => (
    <div className="max-w-2xl">
        <Surface>
            <Watermark text="Map" />
        </Surface>
    </div>
);
