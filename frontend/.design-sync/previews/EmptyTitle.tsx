import { Button, Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "frontend";
import { LockIcon, MapIcon, SearchXIcon } from "lucide-react";

export const NoStagesTracked = () => (
    <Empty>
        <EmptyHeader>
            <EmptyMedia variant="icon">
                <MapIcon />
            </EmptyMedia>
            <EmptyTitle>Chapter 8 has no cleared stages</EmptyTitle>
            <EmptyDescription>Once you clear a stage in Roaring Flare, its drop rates and sanity cost land here.</EmptyDescription>
        </EmptyHeader>
    </Empty>
);

export const ProfilePrivate = () => (
    <Empty>
        <EmptyHeader>
            <EmptyMedia variant="icon">
                <LockIcon />
            </EmptyMedia>
            <EmptyTitle>This profile is private</EmptyTitle>
            <EmptyDescription>The Doctor has not made their roster public. Only their nickname and level are visible.</EmptyDescription>
        </EmptyHeader>
    </Empty>
);

export const WithAction = () => (
    <Empty>
        <EmptyHeader>
            <EmptyMedia variant="icon">
                <SearchXIcon />
            </EmptyMedia>
            <EmptyTitle>No operators match "arts guard"</EmptyTitle>
            <EmptyDescription>Nothing in the roster matches that search on the EN server.</EmptyDescription>
        </EmptyHeader>
        <EmptyContent>
            <Button size="sm" variant="outline">
                Clear search
            </Button>
        </EmptyContent>
    </Empty>
);
