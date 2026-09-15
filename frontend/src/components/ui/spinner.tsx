import { Loader2Icon } from "lucide-react";
import type React from "react";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./spinner.messages";

export function Spinner({ className, ...props }: React.ComponentProps<typeof Loader2Icon>): React.ReactElement {
    const t: TypedT<typeof messages> = useT("common");

    return <Loader2Icon aria-label={t("spinner.loading")} className={cn("animate-spin", className)} role="status" {...props} />;
}
