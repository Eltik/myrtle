import { Checkbox, CheckboxGroupPrimitive, Label } from "frontend";

// CheckboxGroupPrimitive is the unstyled Base UI root re-exported from the DS —
// reach for it when the default vertical stack is the wrong layout.

export const HorizontalRow = () => (
    <CheckboxGroupPrimitive className="flex max-w-md flex-wrap items-center gap-x-5 gap-y-3" defaultValue={["main", "event"]}>
        <span className="font-medium font-sans text-[11px] text-muted-foreground uppercase leading-none tracking-widest">Sections</span>
        {[
            { name: "main", label: "Mainline" },
            { name: "event", label: "Side story" },
            { name: "sss", label: "S.S.S." },
            { name: "cm", label: "CM" },
        ].map((section) => (
            <Label className="cursor-pointer gap-2" key={section.name}>
                <Checkbox name={section.name} />
                <span className="text-sm">{section.label}</span>
            </Label>
        ))}
    </CheckboxGroupPrimitive>
);

export const TwoColumnGrid = () => (
    <CheckboxGroupPrimitive className="grid max-w-md grid-cols-2 gap-x-4 gap-y-2.5" defaultValue={["guard", "sniper", "medic"]}>
        {[
            { name: "guard", label: "Guard" },
            { name: "sniper", label: "Sniper" },
            { name: "defender", label: "Defender" },
            { name: "medic", label: "Medic" },
            { name: "caster", label: "Caster" },
            { name: "specialist", label: "Specialist" },
        ].map((cls) => (
            <Label className="cursor-pointer gap-2 rounded px-1.5 py-1 font-normal hover:bg-accent/50" key={cls.name}>
                <Checkbox name={cls.name} />
                <span className="text-[12.5px]">{cls.label}</span>
            </Label>
        ))}
    </CheckboxGroupPrimitive>
);

export const ParentAndChildren = () => (
    <CheckboxGroupPrimitive allValues={["orirock", "device", "sugar"]} className="flex max-w-xs flex-col gap-2.5" defaultValue={["orirock"]}>
        <Label className="cursor-pointer gap-2.5">
            <Checkbox parent />
            <span className="font-medium text-sm">Track all 1-7 drops</span>
        </Label>
        <div className="flex flex-col gap-2.5 pl-6">
            {[
                { name: "orirock", label: "Orirock Cube" },
                { name: "device", label: "Damaged Device" },
                { name: "sugar", label: "Sugar Substitute" },
            ].map((item) => (
                <Label className="cursor-pointer gap-2.5 font-normal" key={item.name}>
                    <Checkbox name={item.name} />
                    <span className="text-[12.5px]">{item.label}</span>
                </Label>
            ))}
        </div>
    </CheckboxGroupPrimitive>
);
