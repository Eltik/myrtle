import { ChevronDown, ChevronUp } from "lucide-react";
import * as React from "react";
import { Collapsible, CollapsibleContent } from "#/components/ui/collapsible";
import { Label } from "#/components/ui/label";
import { NumberField, NumberFieldDecrement, NumberFieldGroup, NumberFieldIncrement, NumberFieldInput } from "#/components/ui/number-field";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { fromDisplay, type IZoneConfig, type IZoneField, isZoneSet, STACKING_LABEL_KEYS, toDisplay, ZONE_FIELDS } from "../zones";
import type { messages } from "./ZonePanel.messages";
import type { messages as zoneMessages } from "./zones.messages";

type ZoneT = TypedT<typeof messages & typeof zoneMessages>;

interface IZoneFieldsProps {
    zones: IZoneConfig;
    onChange: (patch: Partial<IZoneConfig>) => void;
    /** Changes whenever the values change from outside (hydration, clear), so the uncontrolled inputs remount. */
    resetKey: string | number;
    /** Smaller inputs for the per-card editor. */
    compact?: boolean;
}

const COMMON = ZONE_FIELDS.filter((f) => f.common);
const RARE = ZONE_FIELDS.filter((f) => !f.common);

/** The zone inputs: the common zones always, the rest behind "More buffs" (open when one of them is set). */
export function ZoneFields({ zones, onChange, resetKey, compact = false }: IZoneFieldsProps): React.ReactElement {
    const t: ZoneT = useT("tools");
    const rareSet = RARE.filter((f) => isZoneSet(zones, f.key)).length;
    const [more, setMore] = React.useState(rareSet > 0);
    return (
        <div className="space-y-2">
            <div className="grid grid-cols-2 gap-2">
                {COMMON.map((f) => (
                    <ZoneInput key={`${f.key}-${resetKey}`} field={f} value={zones[f.key]} onChange={onChange} compact={compact} label={t(f.labelKey)} stackingLabel={t(STACKING_LABEL_KEYS[f.stacking])} />
                ))}
            </div>
            <Collapsible open={more} onOpenChange={setMore}>
                <button type="button" onClick={() => setMore((v) => !v)} className="flex w-full cursor-pointer items-center justify-between rounded py-1 text-left font-medium text-[11px] text-muted-foreground hover:text-foreground">
                    <span className="flex items-center gap-1.5">
                        {more ? t("dps.zones.less") : t("dps.zones.more")}
                        {rareSet > 0 && <span className="rounded bg-primary/15 px-1.5 py-0.5 font-mono text-[9.5px] text-primary">{t("dps.zones.moreSet", { count: rareSet })}</span>}
                    </span>
                    {more ? <ChevronUp className="size-3.5" /> : <ChevronDown className="size-3.5" />}
                </button>
                <CollapsibleContent>
                    <div className="grid grid-cols-2 gap-2 pt-1.5">
                        {RARE.map((f) => (
                            <ZoneInput key={`${f.key}-${resetKey}`} field={f} value={zones[f.key]} onChange={onChange} compact={compact} label={t(f.labelKey)} stackingLabel={t(STACKING_LABEL_KEYS[f.stacking])} />
                        ))}
                    </div>
                </CollapsibleContent>
            </Collapsible>
        </div>
    );
}

interface IZoneInputProps {
    field: IZoneField;
    value: number;
    onChange: (patch: Partial<IZoneConfig>) => void;
    compact: boolean;
    label: string;
    stackingLabel: string;
}

function ZoneInput({ field, value, onChange, compact, label, stackingLabel }: IZoneInputProps): React.ReactElement {
    const unset = field.unit === "factor" ? 1 : 0;
    return (
        <NumberField
            defaultValue={toDisplay(field.unit, value)}
            min={field.min}
            max={field.max}
            step={field.step}
            // An emptied input means "unset", which for the factor is 1, not 0 (0 is a 400).
            onValueChange={(v) => onChange({ [field.key]: fromDisplay(field.unit, v ?? unset) })}
            size="sm"
            className="gap-1"
        >
            <Tooltip>
                <TooltipTrigger render={<Label className={`block cursor-help truncate font-medium text-muted-foreground leading-none ${compact ? "text-[10.5px]" : "text-[11px]"}`} />}>{label}</TooltipTrigger>
                <TooltipPopup>{stackingLabel}</TooltipPopup>
            </Tooltip>
            <NumberFieldGroup>
                <NumberFieldDecrement />
                <NumberFieldInput />
                <NumberFieldIncrement />
            </NumberFieldGroup>
        </NumberField>
    );
}
