import { Combobox, ComboboxEmpty, ComboboxInput, ComboboxItem, ComboboxList, ComboboxPopup } from "frontend";
import { MapPin, Search } from "lucide-react";
import type { ReactNode } from "react";

const STAGES = ["1-7 — Vanguard Training", "4-8 — Iron Heart", "S4-1 — Pyrolysis", "JT8-2 — Trials", "CE-6 — Cargo Escort", "LS-6 — Tactical Drill", "AP-5 — Resource Search", "CA-5 — Aerial Threat"];

const Field = ({ children }: { children: ReactNode }) => <div className="w-80 space-y-2">{children}</div>;

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

export const Default = () => (
    <Field>
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="stage-default">
            Stage
        </label>
        <Combobox<string, false> items={STAGES}>
            <ComboboxInput id="stage-default" placeholder="Search 1,240 stages..." />
            <Popup />
        </Combobox>
    </Field>
);

export const WithStartAddon = () => (
    <Field>
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="stage-addon">
            Stage
        </label>
        <Combobox<string, false> items={STAGES}>
            <ComboboxInput id="stage-addon" placeholder="Search 1,240 stages..." startAddon={<MapPin />} />
            <Popup />
        </Combobox>
        <p className="text-[11px] text-muted-foreground leading-snug">Sanity cost and drop rates load once a stage is picked.</p>
    </Field>
);

export const Sizes = () => (
    <div className="flex w-80 flex-col gap-3">
        <Combobox<string, false> items={STAGES}>
            <ComboboxInput size="sm" placeholder="Small — search stages" startAddon={<Search />} />
            <Popup />
        </Combobox>
        <Combobox<string, false> items={STAGES}>
            <ComboboxInput placeholder="Default — search stages" startAddon={<Search />} />
            <Popup />
        </Combobox>
        <Combobox<string, false> items={STAGES}>
            <ComboboxInput size="lg" placeholder="Large — search stages" startAddon={<Search />} />
            <Popup />
        </Combobox>
    </div>
);

export const WithClear = () => (
    <Field>
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="stage-clear">
            Stage
        </label>
        <Combobox<string, false> defaultValue="CE-6 — Cargo Escort" defaultInputValue="CE-6 — Cargo Escort" items={STAGES}>
            <ComboboxInput id="stage-clear" showClear showTrigger={false} placeholder="Search 1,240 stages..." startAddon={<MapPin />} />
            <Popup />
        </Combobox>
    </Field>
);

export const Invalid = () => (
    <Field>
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="stage-invalid">
            Stage
        </label>
        {/* defaultInputValue lives on the root — passing it to ComboboxInput is silently ignored. */}
        <Combobox<string, false> defaultInputValue="9-99" items={STAGES}>
            <ComboboxInput aria-invalid id="stage-invalid" placeholder="Search 1,240 stages..." startAddon={<MapPin />} />
            <Popup />
        </Combobox>
        <p className="text-[11px] text-destructive leading-snug">Stage 9-99 doesn't exist on Global.</p>
    </Field>
);

export const Disabled = () => (
    <Field>
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="stage-disabled">
            Stage
        </label>
        <Combobox<string, false> disabled items={STAGES}>
            <ComboboxInput id="stage-disabled" placeholder="Pick an event first" startAddon={<MapPin />} />
        </Combobox>
    </Field>
);
