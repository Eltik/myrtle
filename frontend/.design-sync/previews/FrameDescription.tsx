import { Frame, FrameDescription, FrameHeader, FramePanel, FrameTitle } from "frontend";

export const UnderTitle = () => (
    <Frame className="max-w-md">
        <FramePanel className="p-0">
            <FrameHeader>
                <FrameTitle>Sanity budget</FrameTitle>
                <FrameDescription>How much sanity the planner is allowed to spend per day when it builds a farming route.</FrameDescription>
            </FrameHeader>
            <div className="border-t px-5 py-4 text-muted-foreground text-sm">
                Current budget: <span className="font-mono font-medium text-foreground tabular-nums">240</span> sanity/day
            </div>
        </FramePanel>
    </Frame>
);

export const AsMetaLine = () => (
    <Frame className="max-w-md">
        <FramePanel className="p-0">
            <FrameHeader>
                <FrameTitle>CE-6 · Cargo Escort</FrameTitle>
                <FrameDescription className="font-mono text-xs tabular-nums">30 sanity · 10,000 LMD · 1.7 clears/min</FrameDescription>
            </FrameHeader>
        </FramePanel>
        <FramePanel className="p-0">
            <FrameHeader>
                <FrameTitle>LS-6 · Tough Siege</FrameTitle>
                <FrameDescription className="font-mono text-xs tabular-nums">30 sanity · 3,000 EXP · 1.4 clears/min</FrameDescription>
            </FrameHeader>
        </FramePanel>
    </Frame>
);

export const LongCopy = () => (
    <Frame className="max-w-sm">
        <FramePanel className="p-0">
            <FrameHeader>
                <FrameTitle>Re-scan base layout</FrameTitle>
                <FrameDescription>
                    We read the room levels and operator assignments straight from your last sync. If you moved operators in-game since then, the optimiser will keep suggesting shifts you have already
                    made.
                </FrameDescription>
            </FrameHeader>
        </FramePanel>
    </Frame>
);
