import { Button, Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "frontend";
import { PackageOpenIcon, PlusIcon, SparklesIcon } from "lucide-react";

export const WithMedia = () => (
    <Empty>
        <EmptyHeader>
            <EmptyMedia variant="icon">
                <PackageOpenIcon />
            </EmptyMedia>
            <EmptyTitle>No Skill Books in the depot</EmptyTitle>
            <EmptyDescription>This Doctor's inventory has no items in the Skill Books category. Resync the profile after a run to pull the latest depot contents.</EmptyDescription>
        </EmptyHeader>
    </Empty>
);

export const TextOnly = () => (
    <Empty>
        <EmptyHeader>
            <EmptyTitle>No public profiles yet</EmptyTitle>
            <EmptyDescription>Public Doctor profiles will appear here as players opt in from their account settings.</EmptyDescription>
        </EmptyHeader>
    </Empty>
);

export const InDashedPanel = () => (
    <Empty className="rounded-lg border border-border border-dashed bg-muted/10">
        <EmptyHeader>
            <EmptyMedia variant="icon">
                <SparklesIcon />
            </EmptyMedia>
            <EmptyTitle>Your workshop is empty</EmptyTitle>
            <EmptyDescription>Create your first tier list to start ranking operators. You can publish it instantly and share it with anyone.</EmptyDescription>
        </EmptyHeader>
        <EmptyContent>
            <Button>
                <PlusIcon />
                Create your first list
            </Button>
        </EmptyContent>
    </Empty>
);
