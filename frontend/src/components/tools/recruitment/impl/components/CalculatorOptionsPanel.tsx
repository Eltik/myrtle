import { ChevronDownIcon } from "lucide-react";
import type * as React from "react";
import { Select, SelectContent, SelectItem, SelectPrimitive, SelectValue, selectTriggerIconClassName, selectTriggerVariants } from "#/components/ui/select";
import { Switch } from "#/components/ui/switch";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { ICalculatorSettings, IRosterViewOptions, OperatorSortMode, ResultLayout } from "../types";
import type { messages } from "./CalculatorOptionsPanel.messages";

interface ICalculatorOptionsPanelProps {
    settings: ICalculatorSettings;
    rosterView: IRosterViewOptions;
    /** Whether a roster can back the overlays; signed out, the roster rows render disabled with a hint. */
    rosterAvailable: boolean;
    layout: ResultLayout;
    onChangeSettings: (patch: Partial<ICalculatorSettings>) => void;
    onChangeRosterView: (patch: Partial<IRosterViewOptions>) => void;
    onChangeLayout: (layout: ResultLayout) => void;
}

type OptionsT = TypedT<typeof messages>;
type OptionsKey = keyof typeof messages & string;

/** Sort order, with the message key each mode's label lives under. `roster` modes read the signed-in roster and are listed only when one is there. */
const SORT_MODES: readonly { value: OperatorSortMode; labelKey: OptionsKey; roster?: boolean }[] = [
    { value: "rarity-desc", labelKey: "recruit.options.sort.rarityDesc" },
    { value: "common-first", labelKey: "recruit.options.sort.commonFirst" },
    { value: "potential-asc", labelKey: "recruit.options.sort.potentialAsc", roster: true },
];

const LAYOUTS: readonly { value: ResultLayout; labelKey: OptionsKey }[] = [
    { value: "compact", labelKey: "recruit.options.layout.compact" },
    { value: "detailed", labelKey: "recruit.options.layout.detailed" },
];

export function CalculatorOptionsPanel({ settings, rosterView, rosterAvailable, layout, onChangeSettings, onChangeRosterView, onChangeLayout }: ICalculatorOptionsPanelProps): React.ReactElement {
    const t: OptionsT = useT("tools");
    const sortModes = SORT_MODES.filter((m) => rosterAvailable || !m.roster);
    const rosterHint = rosterAvailable ? undefined : t("recruit.options.roster.signIn");
    return (
        <div className="flex flex-col gap-3">
            <ToggleRow id="recruit-include-robots" label={t("recruit.options.includeRobots")} checked={settings.includeRobots} onCheckedChange={(includeRobots) => onChangeSettings({ includeRobots })} />
            <ToggleRow id="recruit-include-two-stars" label={t("recruit.options.includeTwoStars")} checked={settings.includeTwoStars} onCheckedChange={(includeTwoStars) => onChangeSettings({ includeTwoStars })} />
            <ToggleRow id="recruit-include-three-stars" label={t("recruit.options.includeThreeStars")} checked={settings.includeThreeStars} onCheckedChange={(includeThreeStars) => onChangeSettings({ includeThreeStars })} />
            <div className="flex flex-col gap-3 border-border/60 border-t pt-3">
                <ToggleRow id="recruit-show-potentials" label={t("recruit.options.showPotentials")} hint={rosterHint} checked={rosterAvailable && rosterView.showPotentials} disabled={!rosterAvailable} onCheckedChange={(showPotentials) => onChangeRosterView({ showPotentials })} />
                <ToggleRow id="recruit-show-next-upgrade" label={t("recruit.options.showNextUpgrade")} checked={rosterAvailable && rosterView.showNextUpgrade} disabled={!rosterAvailable} onCheckedChange={(showNextUpgrade) => onChangeRosterView({ showNextUpgrade })} />
            </div>
            <div className="flex flex-col gap-3 border-border/60 border-t pt-3">
                <SelectRow id="recruit-layout" label={t("recruit.options.layout")} value={layout} options={LAYOUTS} t={t} onValueChange={onChangeLayout} />
                <SelectRow id="recruit-sort" label={t("recruit.options.sort")} placeholder={t("recruit.options.sort.placeholder")} value={settings.operatorSortMode} options={sortModes} t={t} onValueChange={(operatorSortMode) => onChangeSettings({ operatorSortMode })} />
            </div>
        </div>
    );
}

interface IToggleRowProps {
    id: string;
    label: string;
    hint?: string;
    checked: boolean;
    disabled?: boolean;
    onCheckedChange: (value: boolean) => void;
}

function ToggleRow({ id, label, hint, checked, disabled = false, onCheckedChange }: IToggleRowProps): React.ReactElement {
    return (
        <label htmlFor={id} className={cn("flex items-center justify-between gap-3 text-foreground text-sm", disabled ? "cursor-not-allowed text-muted-foreground" : "cursor-pointer")}>
            <span className="flex min-w-0 flex-col">
                <span>{label}</span>
                {hint ? <span className="text-[11px] text-muted-foreground leading-snug">{hint}</span> : null}
            </span>
            <Switch id={id} checked={checked} disabled={disabled} onCheckedChange={onCheckedChange} />
        </label>
    );
}

interface ISelectRowProps<V extends string> {
    id: string;
    label: string;
    placeholder?: string;
    value: V;
    options: readonly { value: V; labelKey: OptionsKey }[];
    t: OptionsT;
    onValueChange: (value: V) => void;
}

function SelectRow<V extends string>({ id, label, placeholder, value, options, t, onValueChange }: ISelectRowProps<V>): React.ReactElement {
    const labelOf = (v: V) => {
        const entry = options.find((o) => o.value === v);
        return entry ? t(entry.labelKey) : v;
    };
    return (
        <div className="flex flex-col gap-1.5">
            <label htmlFor={id} className="font-medium text-foreground text-sm">
                {label}
            </label>
            <Select value={value} onValueChange={(v) => onValueChange(v as V)}>
                <SelectPrimitive.Trigger id={id} data-slot="select-trigger" className={cn(selectTriggerVariants({ size: "sm" }), "w-fit min-w-0")}>
                    <SelectValue placeholder={placeholder}>{(v: string) => labelOf(v as V)}</SelectValue>
                    <SelectPrimitive.Icon data-slot="select-icon">
                        <ChevronDownIcon className={selectTriggerIconClassName} />
                    </SelectPrimitive.Icon>
                </SelectPrimitive.Trigger>
                <SelectContent>
                    {options.map((o) => (
                        <SelectItem key={o.value} value={o.value}>
                            {t(o.labelKey)}
                        </SelectItem>
                    ))}
                </SelectContent>
            </Select>
        </div>
    );
}
