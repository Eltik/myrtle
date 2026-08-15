import { Badge, Button, Frame, FrameDescription, FrameFooter, FrameHeader, FramePanel, FrameTitle } from "frontend";

export const TitleAndDescription = () => (
    <Frame className="max-w-md">
        <FramePanel className="p-0">
            <FrameHeader>
                <FrameTitle>Account linking</FrameTitle>
                <FrameDescription>Connect a Yostar account to import your roster, depot and base.</FrameDescription>
            </FrameHeader>
            <div className="border-t px-5 py-4 text-muted-foreground text-sm">Linked as Dr. Kal'tsit#4417 on the EN server.</div>
        </FramePanel>
    </Frame>
);

export const TitleOnly = () => (
    <Frame className="max-w-md">
        <FramePanel className="p-0">
            <FrameHeader>
                <FrameTitle>Recent pulls</FrameTitle>
            </FrameHeader>
            <div className="border-t px-5 py-4 text-muted-foreground text-sm">120 pulls on Vigilo · 3 six-stars, last at pity 41.</div>
        </FramePanel>
    </Frame>
);

export const HeaderPerSection = () => (
    <Frame className="max-w-md">
        <FramePanel className="p-0">
            <FrameHeader>
                <FrameTitle>Trading post</FrameTitle>
                <FrameDescription>2 slots · 4,320 LMD/day</FrameDescription>
            </FrameHeader>
            <FrameFooter className="flex items-center justify-between border-t">
                <Badge variant="success">Optimal</Badge>
                <Button size="sm" variant="ghost">
                    View shift
                </Button>
            </FrameFooter>
        </FramePanel>
        <FramePanel className="p-0">
            <FrameHeader>
                <FrameTitle>Factory — Originium Shard</FrameTitle>
                <FrameDescription>3 slots · 74 shards/day</FrameDescription>
            </FrameHeader>
            <FrameFooter className="flex items-center justify-between border-t">
                <Badge variant="warning">Swap suggested</Badge>
                <Button size="sm" variant="ghost">
                    View shift
                </Button>
            </FrameFooter>
        </FramePanel>
    </Frame>
);
