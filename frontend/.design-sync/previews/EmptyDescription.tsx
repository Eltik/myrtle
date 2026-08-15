import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "frontend";
import { SearchIcon, TriangleAlertIcon, UsersIcon } from "lucide-react";

export const WithHighlightedQuery = () => (
    <Empty>
        <EmptyHeader>
            <EmptyMedia variant="icon">
                <SearchIcon aria-hidden="true" />
            </EmptyMedia>
            <EmptyTitle>No doctors found</EmptyTitle>
            <EmptyDescription>
                No public profiles match <span className="font-medium text-foreground">"Kal'tsit"</span>. Try a different nickname.
            </EmptyDescription>
        </EmptyHeader>
    </Empty>
);

export const WithInlineLink = () => (
    <Empty>
        <EmptyHeader>
            <EmptyMedia variant="icon">
                <UsersIcon aria-hidden="true" />
            </EmptyMedia>
            <EmptyTitle>No operators in this tier</EmptyTitle>
            <EmptyDescription>
                Drag operators from the pool to rank them, or browse the full <a href="/operators">operator index</a> to find someone specific.
            </EmptyDescription>
        </EmptyHeader>
    </Empty>
);

export const LongCopy = () => (
    <Empty>
        <EmptyHeader>
            <EmptyMedia variant="icon">
                <TriangleAlertIcon aria-hidden="true" />
            </EmptyMedia>
            <EmptyTitle>Depot sync failed</EmptyTitle>
            <EmptyDescription>We could not reach the YoStar servers to refresh your inventory. This usually clears up during maintenance windows — your last successful sync from 6 hours ago is still cached and shown elsewhere on the site.</EmptyDescription>
        </EmptyHeader>
    </Empty>
);
