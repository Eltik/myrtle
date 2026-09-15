import type * as React from "react";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./PullsPlannerTab.messages";

export function PullsPlannerTab(): React.ReactElement {
    const t: TypedT<typeof messages> = useT("tools");
    return (
        <div className="rounded-xl border border-border border-dashed p-10 text-center">
            <p className="font-medium text-[13px] text-foreground">{t("release.pulls.title")}</p>
            <p className="mx-auto mt-1 max-w-md text-[12.5px] text-muted-foreground">{t("release.pulls.desc")}</p>
        </div>
    );
}
