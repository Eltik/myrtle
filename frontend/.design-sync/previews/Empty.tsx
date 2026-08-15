import { Button, Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "frontend";
import { SearchXIcon, TriangleAlertIcon } from "lucide-react";

export const NoResults = () => (
    <Empty>
        <EmptyHeader>
            <EmptyMedia variant="icon">
                <SearchXIcon />
            </EmptyMedia>
            <EmptyTitle>No operators match those filters</EmptyTitle>
            <EmptyDescription>Try widening the rarity range, or clear the class filter to see every operator on this server.</EmptyDescription>
        </EmptyHeader>
        <EmptyContent>
            <Button variant="outline">Clear all filters</Button>
        </EmptyContent>
    </Empty>
);

export const ErrorState = () => (
    <Empty>
        <EmptyHeader>
            <EmptyMedia variant="icon">
                <TriangleAlertIcon />
            </EmptyMedia>
            <EmptyTitle>Recruitment calculator failed to load</EmptyTitle>
            <EmptyDescription>An unexpected error occurred while loading recruitment data. Try again, or check the browser console for details.</EmptyDescription>
        </EmptyHeader>
        <EmptyContent>
            <Button>Try again</Button>
        </EmptyContent>
    </Empty>
);

export const Bare = () => (
    <Empty>
        <EmptyHeader>
            <EmptyTitle>Nothing planned yet</EmptyTitle>
            <EmptyDescription>Operators you add to the planner will show up here with their material costs.</EmptyDescription>
        </EmptyHeader>
    </Empty>
);
