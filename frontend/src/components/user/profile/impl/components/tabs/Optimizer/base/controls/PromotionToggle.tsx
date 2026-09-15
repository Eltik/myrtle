import { Switch } from "#/components/ui/switch";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { useBaseOptimizer } from "../base-context";
import type { messages } from "./PromotionToggle.messages";

const FIELD_ID = "base-ignore-promotion";

export function PromotionToggle() {
    const t: TypedT<typeof messages> = useT("user");
    const api = useBaseOptimizer();

    return (
        <Tooltip>
            <TooltipTrigger
                render={(props) => (
                    <div {...props} className="flex items-center gap-2">
                        <Switch checked={api.ignorePromotion} id={FIELD_ID} onCheckedChange={api.setIgnorePromotion} />
                        <label className="cursor-pointer text-[12px] text-muted-foreground" htmlFor={FIELD_ID}>
                            {t("profile.base.ignorePromotion")}
                        </label>
                    </div>
                )}
            />
            <TooltipPopup className="max-w-64">{t("profile.base.ignorePromotion.tooltip")}</TooltipPopup>
        </Tooltip>
    );
}
