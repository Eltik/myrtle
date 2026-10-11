import type * as React from "react";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { cn } from "#/lib/utils";

interface IBirthdayEmptyProps {
    icon: React.ReactNode;
    title: string;
    description: string;
    className?: string;
}

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
