import { ColorPicker } from "frontend";

const noop = () => {};

export const PresetSelected = () => (
    <div className="w-64">
        <ColorPicker value="#dc4d56" onChange={noop} />
    </div>
);

export const CustomHex = () => (
    <div className="w-64">
        <ColorPicker value="#3f7d5b" onChange={noop} />
    </div>
);

export const InTierSettings = () => (
    <div className="w-80 rounded-2xl border border-border bg-popover p-4 shadow-md">
        <p className="m-0 font-bold font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.2em]">Tier settings</p>
        <h3 className="m-0 mt-2 mb-4 font-bold font-sans text-foreground text-lg leading-tight tracking-tight">S+ — colour</h3>
        <ColorPicker value="#5aa9d9" onChange={noop} />
    </div>
);
