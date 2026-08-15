import { TogglePrimitive } from "frontend";
import { CheckIcon, PinIcon } from "lucide-react";

// TogglePrimitive is the unstyled Base UI root that `Toggle` wraps. Reach for it
// when a surface needs pressed-state semantics but its own visual language —
// the roster's rarity chips and the planner's tag pills both do.

/** Rarity chips: the pressed state is expressed with the DS's own tokens, not Toggle's. */
export const RarityChips = () => (
    <div className="flex flex-wrap items-center gap-2">
        {[
            { stars: "6★", pressed: true },
            { stars: "5★", pressed: true },
            { stars: "4★", pressed: false },
            { stars: "3★", pressed: false },
        ].map((r) => (
            <TogglePrimitive
                aria-label={`${r.stars} operators`}
                className="inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-full border border-input bg-background px-3 font-medium text-sm outline-none transition-colors data-pressed:bg-accent data-pressed:text-accent-foreground"
                defaultPressed={r.pressed}
                key={r.stars}
            >
                {r.stars}
            </TogglePrimitive>
        ))}
    </div>
);

/** Recruitment tags: a check appears once the tag is selected. */
export const RecruitmentTags = () => (
    <div className="flex w-full max-w-md flex-wrap items-center gap-2">
        {[
            { tag: "Top Operator", pressed: true },
            { tag: "Senior Operator", pressed: false },
            { tag: "Defense", pressed: true },
            { tag: "Healing", pressed: false },
            { tag: "Slow", pressed: false },
        ].map((t) => (
            <TogglePrimitive
                className="inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-lg border border-input bg-background px-2.5 text-sm outline-none transition-colors data-pressed:bg-input/64 data-pressed:text-accent-foreground"
                defaultPressed={t.pressed}
                key={t.tag}
            >
                {t.pressed ? <CheckIcon className="size-3.5" /> : null}
                {t.tag}
            </TogglePrimitive>
        ))}
    </div>
);

/** Pinned rows in the leaderboard sidebar — icon-only, pressed and unpressed. */
export const PinRow = () => (
    <div className="flex w-full max-w-md flex-col gap-2">
        {[
            { name: "Kal'tsit Enjoyer", pinned: true },
            { name: "Rhodes Logistics", pinned: false },
        ].map((d) => (
            <div className="flex items-center justify-between gap-3 rounded-lg border bg-card px-3 py-2" key={d.name}>
                <span className="font-medium text-sm">{d.name}</span>
                <TogglePrimitive
                    aria-label={`Pin ${d.name}`}
                    className="inline-flex size-8 cursor-pointer items-center justify-center rounded-md text-muted-foreground outline-none transition-colors data-pressed:bg-input/64 data-pressed:text-accent-foreground"
                    defaultPressed={d.pinned}
                >
                    <PinIcon className="size-4" />
                </TogglePrimitive>
            </div>
        ))}
    </div>
);
