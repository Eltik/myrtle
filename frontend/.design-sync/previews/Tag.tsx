import { Tag } from "frontend";

// The release planner's mono kicker chip: a kind or archetype label beside a row
// name. Muted by default; the schedule tints the text with the row kind's hue.
export const Default = () => (
    <div className="flex flex-wrap items-center gap-2 p-4">
        <Tag>Side story</Tag>
        <Tag>Rerun</Tag>
        <Tag>Limited</Tag>
    </div>
);

// Beside a row title, the way `ScheduleDetail` uses it: the kind chip carries the
// kind's colour, the archetype chip stays neutral.
export const KindTinted = () => (
    <div className="flex flex-col gap-2 p-4">
        <span className="flex items-center gap-2 font-sans font-semibold text-[14px] text-foreground">
            Adventure That Cannot Wait for the Sun - Rerun
            <Tag className="text-sky-600 dark:text-sky-400">Event</Tag>
            <Tag>Side story</Tag>
        </span>
        <span className="flex items-center gap-2 font-sans font-semibold text-[14px] text-foreground">
            Cantilena Puppae
            <Tag className="text-yellow-600 dark:text-yellow-300">Banner</Tag>
            <Tag>Limited</Tag>
        </span>
        <span className="flex items-center gap-2 font-sans font-semibold text-[14px] text-foreground">
            Rhodes Island Pharmaceuticals Summer Collection
            <Tag className="text-pink-600 dark:text-pink-400">Skin</Tag>
        </span>
    </div>
);
