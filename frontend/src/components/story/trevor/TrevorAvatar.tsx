import type React from "react";
import { Avatar, AvatarFallback } from "#/components/ui/avatar";

/** Trevor's avatar beside its chat messages: a "T" on the primary colour. */
export function TrevorAvatar(): React.ReactElement {
    return (
        <Avatar aria-hidden className="size-7 bg-primary font-semibold text-primary-foreground text-xs">
            <AvatarFallback className="bg-primary text-primary-foreground">T</AvatarFallback>
        </Avatar>
    );
}
