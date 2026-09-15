import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./ComingSoon.messages";
import type { IOptimizerDef } from "./optimizers";
import type { messages as optimizerMessages } from "./optimizers.messages";

export function ComingSoon({ def }: { def: IOptimizerDef }) {
    /** The optimizer's own name and blurb are declared in `optimizers.messages.ts`. */
    const t: TypedT<typeof messages & typeof optimizerMessages> = useT("user");
    return (
        <div className="rounded-xl border border-border border-dashed p-10 text-center">
            <p className="font-medium text-[13px] text-foreground">{t("profile.optimizer.comingSoon", { name: t(def.labelKey) })}</p>
            <p className="mx-auto mt-1 max-w-md text-[12.5px] text-muted-foreground">{t(def.blurbKey)}</p>
        </div>
    );
}
