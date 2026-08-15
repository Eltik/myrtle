import { Button, Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "frontend";
import { PlusIcon, RefreshCwIcon, SearchXIcon, WifiOffIcon } from "lucide-react";

export const SingleAction = () => (
    <Empty>
        <EmptyHeader>
            <EmptyMedia variant="icon">
                <SearchXIcon />
            </EmptyMedia>
            <EmptyTitle>No recruitment tags selected</EmptyTitle>
            <EmptyDescription>Pick the tags shown in your recruitment panel to see which operators are guaranteed.</EmptyDescription>
        </EmptyHeader>
        <EmptyContent>
            <Button variant="outline">Reset tags</Button>
        </EmptyContent>
    </Empty>
);

export const PrimaryAndSecondary = () => (
    <Empty>
        <EmptyHeader>
            <EmptyMedia variant="icon">
                <PlusIcon />
            </EmptyMedia>
            <EmptyTitle>Nothing planned yet</EmptyTitle>
            <EmptyDescription>Add operators to the planner to see the materials and LMD their promotions will cost.</EmptyDescription>
        </EmptyHeader>
        <EmptyContent>
            <div className="flex items-center gap-2">
                <Button>Add operator</Button>
                <Button variant="outline">Import from profile</Button>
            </div>
        </EmptyContent>
    </Empty>
);

export const ActionWithHint = () => (
    <Empty>
        <EmptyHeader>
            <EmptyMedia variant="icon">
                <WifiOffIcon />
            </EmptyMedia>
            <EmptyTitle>Could not load stage drops</EmptyTitle>
            <EmptyDescription>The drop table for 1-7 failed to load. Retrying usually fixes it.</EmptyDescription>
        </EmptyHeader>
        <EmptyContent>
            <Button>
                <RefreshCwIcon />
                Try again
            </Button>
            <p className="text-muted-foreground text-xs">Last successful fetch: 14 minutes ago</p>
        </EmptyContent>
    </Empty>
);
