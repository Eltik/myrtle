import { Combobox, ComboboxClear, ComboboxEmpty, ComboboxInput, ComboboxItem, ComboboxList, ComboboxPopup } from "frontend";
import { MapPin, X } from "lucide-react";

const STAGES = ["1-7 — Vanguard Training", "4-8 — Iron Heart", "S4-1 — Pyrolysis", "CE-6 — Cargo Escort", "LS-6 — Tactical Drill", "AP-5 — Resource Search"];

const Popup = () => (
    <ComboboxPopup>
        <ComboboxEmpty>No matching stages.</ComboboxEmpty>
        <ComboboxList>
            {(stage: string) => (
                <ComboboxItem key={stage} value={stage}>
                    {stage}
                </ComboboxItem>
            )}
        </ComboboxList>
    </ComboboxPopup>
);

export const InsideInput = () => (
    <div className="w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="clear-input">
            Stage
        </label>
        <Combobox<string, false> defaultValue="CE-6 — Cargo Escort" defaultInputValue="CE-6 — Cargo Escort" items={STAGES}>
            <ComboboxInput id="clear-input" showClear showTrigger={false} placeholder="Search 1,240 stages..." startAddon={<MapPin />} />
            <Popup />
        </Combobox>
        <p className="text-[11px] text-muted-foreground leading-snug">Clearing the stage resets the sanity and drop-rate panels below.</p>
    </div>
);

export const ClearBesideField = () => (
    <div className="w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="clear-beside">
            Stage
        </label>
        <Combobox<string, false> defaultValue="S4-1 — Pyrolysis" defaultInputValue="S4-1 — Pyrolysis" items={STAGES}>
            <div className="flex items-center gap-2">
                <ComboboxInput id="clear-beside" placeholder="Search 1,240 stages..." startAddon={<MapPin />} />
                <ComboboxClear className="inline-flex h-9 shrink-0 cursor-pointer items-center gap-1 rounded-lg border border-input px-2.5 font-medium text-muted-foreground text-xs outline-none hover:bg-accent hover:text-accent-foreground">
                    <X className="size-3.5" />
                    Reset
                </ComboboxClear>
            </div>
            <Popup />
        </Combobox>
    </div>
);

export const EmptyField = () => (
    <div className="w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="clear-empty">
            Stage
        </label>
        <Combobox<string, false> items={STAGES}>
            <ComboboxInput id="clear-empty" showClear showTrigger={false} placeholder="Search 1,240 stages..." startAddon={<MapPin />} />
            <Popup />
        </Combobox>
        <p className="text-[11px] text-muted-foreground leading-snug">With nothing typed the clear button stays out of the way.</p>
    </div>
);
