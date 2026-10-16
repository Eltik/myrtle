"use client";

import type { ReactElement, ReactNode } from "react";
import { Children } from "react";
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { Skeleton } from "#/components/ui/skeleton";
import { cn } from "#/lib/utils";
import { ChatMessageAvatar, ChatMessageContent, ChatMessageHeader, ChatMessageLayout } from "./chat-message-layout";

export type ChatEmptyStateProps = {
    title: ReactNode;
    description: ReactNode;
    icon: ReactNode;
    children?: ReactNode;
};

export function ChatEmptyState({ title, description, icon, children }: ChatEmptyStateProps): ReactElement {
    return (
        <Empty role="status" className="min-h-64 gap-4 py-10 md:py-10">
            <EmptyHeader>
                <EmptyMedia variant="icon" className="mb-2 bg-popover">
                    {icon}
                </EmptyMedia>
                <EmptyTitle>{title}</EmptyTitle>
                <EmptyDescription>{description}</EmptyDescription>
            </EmptyHeader>
            {Children.toArray(children).length > 0 && <EmptyContent>{children}</EmptyContent>}
        </Empty>
    );
}

const LOADING_ROWS = [
    { id: "first", side: "incoming" },
    { id: "second", side: "outgoing" },
    { id: "third", side: "incoming" },
    { id: "fourth", side: "outgoing" },
] as const;

export function ChatLoadingState({ label, children }: { label: string; children?: ReactNode }): ReactElement {
    return (
        // biome-ignore lint/a11y/useSemanticElements: coss registry markup kept verbatim; a live status region, not form output
        <div role="status" className="flex flex-col gap-5 py-4">
            <span className="sr-only">{label}</span>
            {LOADING_ROWS.map(({ side, id }) => (
                <ChatMessageLayout key={id} align={side === "outgoing" ? "end" : "start"} aria-hidden="true">
                    {side === "incoming" && (
                        <>
                            <ChatMessageHeader>
                                <Skeleton className="h-3 w-20 motion-reduce:animate-none" />
                            </ChatMessageHeader>
                            <ChatMessageAvatar>
                                <Skeleton className="size-7 rounded-full motion-reduce:animate-none" />
                            </ChatMessageAvatar>
                        </>
                    )}
                    <ChatMessageContent>
                        <Skeleton className={cn("h-11 w-56 max-w-[83%] rounded-2xl motion-reduce:animate-none", side === "incoming" ? "rounded-ss-md" : "rounded-se-md")} />
                        <Skeleton className="mx-1 h-2 w-9 motion-reduce:animate-none" />
                    </ChatMessageContent>
                </ChatMessageLayout>
            ))}
            {children}
        </div>
    );
}

export function ChatStatusBar({ title, description, action }: { title: ReactNode; description: ReactNode; action?: ReactNode }): ReactElement {
    return (
        // biome-ignore lint/a11y/useSemanticElements: coss registry markup kept verbatim; a live status region, not form output
        <div role="status" className="flex shrink-0 items-center justify-between gap-3 border-border/60 border-y bg-muted/72 px-5 py-2.5">
            <div className="text-xs">
                <p className="m-0 font-medium">{title}</p>
                <p className="m-0 mt-1 text-muted-foreground">{description}</p>
            </div>
            {action}
        </div>
    );
}
