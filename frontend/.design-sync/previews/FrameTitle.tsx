import { Badge, Frame, FrameDescription, FrameHeader, FramePanel, FrameTitle } from "frontend";

export const InHeader = () => (
    <Frame className="max-w-md">
        <FramePanel className="p-0">
            <FrameHeader>
                <FrameTitle>Chapter 8 — Roaring Flare</FrameTitle>
                <FrameDescription>14 stages, 3 challenge modes.</FrameDescription>
            </FrameHeader>
            <div className="border-t px-5 py-4 text-muted-foreground text-sm">
                First-clear rewards include <span className="font-medium text-foreground">2× Chip Catalyst</span>.
            </div>
        </FramePanel>
    </Frame>
);

export const WithTrailingBadge = () => (
    <Frame className="max-w-md">
        <FramePanel className="p-0">
            <FrameHeader>
                <FrameTitle className="flex items-center justify-between gap-2">
                    Global tier list
                    <Badge variant="secondary">v12</Badge>
                </FrameTitle>
                <FrameDescription>Community-voted rankings, updated every Monday.</FrameDescription>
            </FrameHeader>
            <div className="border-t px-5 py-4 text-muted-foreground text-sm">1,284 placements across 6 tiers, contributed by 312 doctors.</div>
        </FramePanel>
    </Frame>
);

export const Repeated = () => (
    <Frame className="max-w-md">
        <FramePanel className="p-0">
            <FrameHeader>
                <FrameTitle>Mlynar</FrameTitle>
                <FrameDescription>E2 60 → E2 90 · S3M3</FrameDescription>
            </FrameHeader>
        </FramePanel>
        <FramePanel className="p-0">
            <FrameHeader>
                <FrameTitle>Muelsyse</FrameTitle>
                <FrameDescription>E1 55 → E2 40 · S2M3</FrameDescription>
            </FrameHeader>
        </FramePanel>
        <FramePanel className="p-0">
            <FrameHeader>
                <FrameTitle>Texas the Omertosa</FrameTitle>
                <FrameDescription>E2 30 → E2 70 · S3M3</FrameDescription>
            </FrameHeader>
        </FramePanel>
    </Frame>
);
