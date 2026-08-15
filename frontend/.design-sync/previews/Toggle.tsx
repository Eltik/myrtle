import { Toggle } from "frontend";
import { BoldIcon, EyeOffIcon, ItalicIcon, ListIcon, PinIcon, StarIcon } from "lucide-react";

/** The two variants: borderless by default, bordered with `variant="outline"`. */
export const Variants = () => (
    <div className="flex flex-wrap items-center gap-3">
        <Toggle aria-label="Bold" defaultPressed>
            <BoldIcon />
        </Toggle>
        <Toggle aria-label="Italic">
            <ItalicIcon />
        </Toggle>
        <Toggle aria-label="Owned only" defaultPressed variant="outline">
            <StarIcon />
            Owned only
        </Toggle>
        <Toggle aria-label="Hide unreleased" variant="outline">
            <EyeOffIcon />
            Hide unreleased
        </Toggle>
    </div>
);

/** Three sizes, shown pressed so the active surface reads. */
export const Sizes = () => (
    <div className="flex flex-wrap items-center gap-3">
        <Toggle aria-label="Small" defaultPressed size="sm" variant="outline">
            <PinIcon />
            Small
        </Toggle>
        <Toggle aria-label="Default" defaultPressed variant="outline">
            <PinIcon />
            Default
        </Toggle>
        <Toggle aria-label="Large" defaultPressed size="lg" variant="outline">
            <PinIcon />
            Large
        </Toggle>
    </div>
);

/** Pressed, unpressed and disabled side by side. */
export const States = () => (
    <div className="flex flex-wrap items-center gap-3">
        <Toggle aria-label="Pressed" defaultPressed variant="outline">
            Pressed
        </Toggle>
        <Toggle aria-label="Unpressed" variant="outline">
            Unpressed
        </Toggle>
        <Toggle aria-label="Pressed and disabled" defaultPressed disabled variant="outline">
            Pressed · disabled
        </Toggle>
        <Toggle aria-label="Disabled" disabled variant="outline">
            Disabled
        </Toggle>
    </div>
);

/** Icon-only toggles in a formatting strip — the markdown editor's shape. */
export const IconOnly = () => (
    <div className="flex w-fit items-center gap-0.5 rounded-xl border bg-card p-1">
        <Toggle aria-label="Bold" defaultPressed>
            <BoldIcon />
        </Toggle>
        <Toggle aria-label="Italic">
            <ItalicIcon />
        </Toggle>
        <Toggle aria-label="Bullet list">
            <ListIcon />
        </Toggle>
    </div>
);
