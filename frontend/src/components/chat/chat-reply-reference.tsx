"use client";

import { CornerDownRightIcon } from "lucide-react";
import type { ComponentProps, ReactElement } from "react";
import { cn } from "#/lib/utils";

export type ChatReplyReferenceProps = Omit<ComponentProps<"button">, "children"> & {
    author: string;
    preview: string;
};

export function ChatReplyReference({ author, preview, className, ...props }: ChatReplyReferenceProps): ReactElement {
    return (
        <button
            type="button"
            data-slot="chat-reply-reference"
            className={cn("group/reply-reference flex min-w-0 items-start gap-1.5 text-start text-xs outline-none transition-colors focus-visible:rounded-sm focus-visible:text-primary focus-visible:ring-2 focus-visible:ring-current enabled:hover:text-primary motion-reduce:transition-none", className)}
            {...props}
        >
            <CornerDownRightIcon aria-hidden="true" className="mt-0.5 size-3.5 shrink-0 opacity-64 transition-opacity group-hover/reply-reference:opacity-100 group-focus-visible/reply-reference:opacity-100" />
            <span className="flex min-w-0 flex-col gap-0.5">
                <span className="font-medium">{author}</span>
                <span className="line-clamp-2 text-foreground/80">{preview}</span>
            </span>
        </button>
    );
}
