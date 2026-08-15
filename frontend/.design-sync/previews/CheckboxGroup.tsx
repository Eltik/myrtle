import { Checkbox, CheckboxGroup, Label } from "frontend";

const FIELDS = [
    { name: "name", label: "Operator name" },
    { name: "rarity", label: "Rarity" },
    { name: "elite", label: "Elite / level" },
    { name: "module", label: "Module stage" },
    { name: "trust", label: "Trust" },
];

export const ExportFields = () => (
    <CheckboxGroup className="max-w-xs" defaultValue={["name", "rarity", "elite"]}>
        <p className="font-medium font-sans text-[11px] text-muted-foreground uppercase leading-none tracking-widest">Columns to export</p>
        {FIELDS.map((field) => (
            <Label className="cursor-pointer gap-2.5" key={field.name}>
                <Checkbox name={field.name} />
                <span className="text-sm">{field.label}</span>
            </Label>
        ))}
    </CheckboxGroup>
);

export const WithParentCheckbox = () => (
    <CheckboxGroup allValues={FIELDS.map((f) => f.name)} className="max-w-xs" defaultValue={["name", "rarity"]}>
        <Label className="cursor-pointer gap-2.5">
            <Checkbox parent />
            <span className="font-medium text-sm">All operator columns</span>
            <span className="font-mono text-muted-foreground text-xs tabular-nums">2/5</span>
        </Label>
        <div className="flex flex-col gap-3 pl-6">
            {FIELDS.map((field) => (
                <Label className="cursor-pointer gap-2.5 font-normal" key={field.name}>
                    <Checkbox name={field.name} />
                    <span className="text-[12.5px]">{field.label}</span>
                </Label>
            ))}
        </div>
    </CheckboxGroup>
);

export const Disabled = () => (
    <CheckboxGroup className="max-w-xs" defaultValue={["cn"]} disabled>
        <p className="font-medium font-sans text-[11px] text-muted-foreground uppercase leading-none tracking-widest">Servers (import in progress)</p>
        {[
            { name: "en", label: "EN — Yostar" },
            { name: "cn", label: "CN — Hypergryph" },
            { name: "jp", label: "JP — Yostar" },
        ].map((server) => (
            <Label className="gap-2.5 text-muted-foreground" key={server.name}>
                <Checkbox name={server.name} />
                <span className="text-sm">{server.label}</span>
            </Label>
        ))}
    </CheckboxGroup>
);
