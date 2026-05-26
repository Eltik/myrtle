import type * as React from "react";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { cn } from "#/lib/utils";

interface IBirthdayEmptyProps {
    icon: React.ReactNode;
    title: string;
    description: string;
    /** Override padding for tighter layouts. */
    className?: string;
}

/** Shared empty state for the birthday tool. */
export function BirthdayEmpty({ icon, title, description, className }: IBirthdayEmptyProps): React.ReactElement {
    return (
        <Empty className={cn(className)}>
            <EmptyHeader>
                <EmptyMedia variant="icon">{icon}</EmptyMedia>
                <EmptyTitle>{title}</EmptyTitle>
                <EmptyDescription>{description}</EmptyDescription>
            </EmptyHeader>
        </Empty>
    );
}
