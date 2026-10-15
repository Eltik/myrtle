import { PresetRow } from "frontend";
import { type ReactNode, useEffect, useRef } from "react";

// Named presets in the bulk-add dialog: a select to load one, a trash button
// for the chosen one, and a name field with "Save preset" (it turns into
// "Replace preset" when the name matches an existing one). Each action saves
// straight away, independent of the dialog's own Save.
//
// The preset list is a server query, which the design-system bundle cannot
// reach, so the select opens on "No saved presets yet."; what renders here is
// the row at rest.

const target = { elite: 2, level: 90, skill_level: 7, masteries: [0, 0, 3] as [number, number, number], module_stage: 3, display_on_profile: false };
const noop = () => undefined;

/** At rest: nothing chosen, so delete is disabled; an empty name disables Save. */
export const AtRest = () => (
    <div className="max-w-md">
        <PresetRow target={target} onLoad={noop} />
    </div>
);

/**
 * Types into the name field the way a user would: React only sees a value set
 * through the native setter plus an input event, after Base UI has mounted.
 */
function TypeName({ value, children }: { value: string; children: ReactNode }) {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        let f2 = 0;
        const f1 = requestAnimationFrame(() => {
            f2 = requestAnimationFrame(() => {
                const input = ref.current?.querySelector<HTMLInputElement>('input[placeholder="Preset name"]');
                if (!input) return;
                Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set?.call(input, value);
                input.dispatchEvent(new Event("input", { bubbles: true }));
            });
        });
        return () => {
            cancelAnimationFrame(f1);
            cancelAnimationFrame(f2);
        };
    }, [value]);
    return (
        <div className="max-w-md" ref={ref}>
            {children}
        </div>
    );
}

/** A name typed: "Save preset" is live. */
export const NameTyped = () => (
    <TypeName value="IS5 endgame">
        <PresetRow target={target} onLoad={noop} />
    </TypeName>
);
