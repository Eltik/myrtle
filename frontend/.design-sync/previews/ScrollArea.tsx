import { Badge, ScrollArea } from "frontend";

const SKILLS = [
    { name: "Dispersion", meta: "Auto recovery · Manual trigger", mastery: "M3" },
    { name: "Blood Ripper", meta: "Auto recovery · Auto trigger", mastery: "M3" },
    { name: "Undying Rage", meta: "Auto recovery · Manual trigger", mastery: "M1" },
    { name: "Sanguine Armament", meta: "Offensive recovery · Manual", mastery: "—" },
    { name: "Ashen Rime", meta: "Auto recovery · Manual trigger", mastery: "—" },
    { name: "Tidal Surge", meta: "Offensive recovery · Manual", mastery: "—" },
];

const STAGES = ["1-7", "1-12", "4-6", "S4-1", "6-16", "7-18", "8-11", "9-17", "10-8", "11-14", "12-17", "13-16"];

/** A capped skill list — the roster card's Skills accordion panel. */
export const SkillList = () => (
    <div className="h-64 w-80 rounded-lg border bg-card p-2">
        <ScrollArea>
            <div className="flex flex-col gap-2 pr-3">
                {SKILLS.map((skill) => (
                    <div className="flex items-center justify-between gap-2 rounded-md border bg-background px-3 py-2" key={skill.name}>
                        <span className="min-w-0">
                            <span className="block truncate font-medium text-foreground text-sm leading-tight">{skill.name}</span>
                            <span className="block truncate text-muted-foreground text-xs">{skill.meta}</span>
                        </span>
                        <Badge size="default" variant="outline">
                            {skill.mastery}
                        </Badge>
                    </div>
                ))}
            </div>
        </ScrollArea>
    </div>
);

/** `scrollFade` + `scrollbarGutter` — the combobox/autocomplete popup treatment. */
export const FadedList = () => (
    <div className="h-64 w-72 rounded-lg border bg-popover p-2">
        <ScrollArea scrollbarGutter scrollFade>
            <div className="flex flex-col">
                {["Młynar", "Skadi", "Texas", "Eyjafjalla", "Amiya", "Heidi", "Texas the Omertosa", "Muelsyse", "Surtr", "Thorns"].map((name) => (
                    <span className="rounded-md px-2 py-1.5 text-foreground text-sm" key={name}>
                        {name}
                    </span>
                ))}
            </div>
        </ScrollArea>
    </div>
);

/** Horizontal overflow — the stage strip under an operator's clear history. */
export const HorizontalStrip = () => (
    <div className="h-20 w-80 rounded-lg border bg-card p-2">
        <ScrollArea>
            <div className="flex w-max gap-2 pb-3">
                {STAGES.map((stage) => (
                    <span className="rounded-md border bg-background px-3 py-2 font-mono text-foreground text-sm tabular-nums" key={stage}>
                        {stage}
                    </span>
                ))}
            </div>
        </ScrollArea>
    </div>
);

/** No overflow — the viewport sits flush and no scrollbar is ever mounted. */
export const NoOverflow = () => (
    <div className="h-64 w-80 rounded-lg border bg-card p-2">
        <ScrollArea>
            <div className="flex flex-col gap-2 pr-3">
                {SKILLS.slice(0, 2).map((skill) => (
                    <div className="flex items-center justify-between gap-2 rounded-md border bg-background px-3 py-2" key={skill.name}>
                        <span className="min-w-0">
                            <span className="block truncate font-medium text-foreground text-sm leading-tight">{skill.name}</span>
                            <span className="block truncate text-muted-foreground text-xs">{skill.meta}</span>
                        </span>
                        <Badge size="default" variant="outline">
                            {skill.mastery}
                        </Badge>
                    </div>
                ))}
            </div>
        </ScrollArea>
    </div>
);
