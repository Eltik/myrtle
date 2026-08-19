import { Switch } from "#/components/ui/switch";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useBaseOptimizer } from "../base-context";

const FIELD_ID = "base-ignore-promotion";

export function PromotionToggle() {
    const api = useBaseOptimizer();

    return (
        <Tooltip>
            <TooltipTrigger
                render={(props) => (
                    <div {...props} className="flex items-center gap-2">
                        <Switch checked={api.ignorePromotion} id={FIELD_ID} onCheckedChange={api.setIgnorePromotion} />
                        <label className="cursor-pointer text-[12px] text-muted-foreground" htmlFor={FIELD_ID}>
                            Ignore promotion
                        </label>
                    </div>
                )}
            />
            <TooltipPopup className="max-w-64">Plan with every operator&rsquo;s highest base skills, even the ones they are not promoted far enough to use. Skills you have not unlocked yet show greyed out.</TooltipPopup>
        </Tooltip>
    );
}
