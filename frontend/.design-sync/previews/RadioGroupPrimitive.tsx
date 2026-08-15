import { Radio, RadioGroupPrimitive } from "frontend";

/** The unstyled Base UI root: no layout of its own, so the grid is yours. */
export const CustomGrid = () => (
    <RadioGroupPrimitive className="grid w-80 grid-cols-2 gap-2" defaultValue="1440">
        {[
            { value: "720", label: "720 × 720", desc: "Fast, forum-friendly" },
            { value: "1080", label: "1080 × 1080", desc: "Default chibi export" },
            { value: "1440", label: "1440 × 1440", desc: "Sharp on retina" },
            { value: "2160", label: "2160 × 2160", desc: "Slow, large file" },
        ].map((res) => (
            // biome-ignore lint/a11y/noLabelWithoutControl: Radio renders the input
            <label className="flex cursor-pointer items-start gap-2 rounded-md border bg-card px-3 py-2 hover:bg-accent" key={res.value}>
                <Radio className="mt-0.5" value={res.value} />
                <span className="flex flex-col gap-0.5">
                    <span className="font-mono font-medium text-foreground text-sm leading-none tabular-nums">{res.label}</span>
                    <span className="text-muted-foreground text-xs">{res.desc}</span>
                </span>
            </label>
        ))}
    </RadioGroupPrimitive>
);

/** A divided menu list — the shape the mobile drawer builds on top of the primitive. */
export const MenuList = () => (
    <RadioGroupPrimitive className="flex w-72 flex-col divide-y rounded-lg border bg-card" defaultValue="rarity">
        {[
            { value: "rarity", label: "Rarity" },
            { value: "class", label: "Class" },
            { value: "release", label: "Release date" },
        ].map((sort) => (
            // biome-ignore lint/a11y/noLabelWithoutControl: Radio renders the input
            <label className="flex cursor-pointer items-center justify-between gap-2 px-3 py-2.5" key={sort.value}>
                <span className="text-foreground text-sm leading-none">{sort.label}</span>
                <Radio value={sort.value} />
            </label>
        ))}
    </RadioGroupPrimitive>
);

/** `name` and `required` make the primitive a real form control. */
export const InAForm = () => (
    <form className="flex w-72 flex-col gap-3">
        <span className="font-medium text-foreground text-sm">Default skill on import</span>
        <RadioGroupPrimitive className="flex flex-col gap-3" defaultValue="s3" name="default_skill" required>
            {[
                { value: "s1", label: "Skill 1" },
                { value: "s2", label: "Skill 2" },
                { value: "s3", label: "Skill 3" },
            ].map((skill) => (
                // biome-ignore lint/a11y/noLabelWithoutControl: Radio renders the input
                <label className="flex cursor-pointer items-center gap-2.5" key={skill.value}>
                    <Radio value={skill.value} />
                    <span className="text-foreground text-sm leading-none">{skill.label}</span>
                </label>
            ))}
        </RadioGroupPrimitive>
    </form>
);
