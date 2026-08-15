import { Badge, Frame, FramePanel } from "frontend";

const DROPS = [
    { rate: "42%", stage: "1-7", tier: "Orirock" },
    { rate: "28%", stage: "S4-1", tier: "Sugar Substitute" },
    { rate: "19%", stage: "CE-6", tier: "Polyester" },
];

export const Single = () => (
    <Frame className="max-w-sm">
        <FramePanel>
            <p className="font-semibold text-sm">Bipolar Nanoflake</p>
            <p className="mt-1 text-muted-foreground text-sm">T5 · crafted from 2× Polymerization Preparation and 1× Manganese Trihydrate.</p>
        </FramePanel>
    </Frame>
);

export const Stacked = () => (
    <Frame className="max-w-md">
        {DROPS.map((drop) => (
            <FramePanel key={drop.stage}>
                <div className="flex items-center justify-between gap-4">
                    <div className="min-w-0">
                        <p className="font-mono font-medium text-sm">{drop.stage}</p>
                        <p className="truncate text-muted-foreground text-xs">{drop.tier}</p>
                    </div>
                    <span className="font-mono text-muted-foreground text-sm tabular-nums">{drop.rate}</span>
                </div>
            </FramePanel>
        ))}
    </Frame>
);

export const PanelWithDivider = () => (
    <Frame className="max-w-md">
        <FramePanel className="p-0">
            <div className="flex items-center justify-between gap-4 px-5 py-4">
                <span className="font-medium text-sm">Annihilation cap</span>
                <Badge variant="info">Weekly</Badge>
            </div>
            <div className="flex items-center justify-between gap-4 border-t px-5 py-4">
                <span className="text-muted-foreground text-sm">Collected</span>
                <span className="font-mono text-sm tabular-nums">1,800 / 1,800</span>
            </div>
        </FramePanel>
    </Frame>
);
