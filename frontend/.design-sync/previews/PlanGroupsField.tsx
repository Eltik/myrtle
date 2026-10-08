import { PlanGroupsField } from "frontend";
import { type ReactNode, useEffect, useRef, useState } from "react";

// The plan dialog's group multi-select. Closed, the input shows the picked
// groups comma-joined; open, it becomes the search box over a list led by
// "Create new group", each row a tick, the name, and rename / delete buttons.
// Those two use native prompt/confirm dialogs, so no story touches them.

const GROUPS = ["IS5 core", "Annihilation team", "Contingency Contract", "Base shift 2"];

/** Presses the chevron trigger after Base UI has wired it, which opens the popup as a user's press does. */
function OpenOnMount({ children }: { children: ReactNode }) {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        let f2 = 0;
        const f1 = requestAnimationFrame(() => {
            f2 = requestAnimationFrame(() => {
                const trigger = ref.current?.querySelector<HTMLElement>('[data-slot="combobox-trigger"]');
                if (!trigger) return;
                // Base UI opens on the press, not the click: send the whole sequence.
                for (const type of ["pointerdown", "mousedown", "pointerup", "mouseup", "click"]) {
                    const Ctor = type.startsWith("pointer") ? PointerEvent : MouseEvent;
                    trigger.dispatchEvent(new Ctor(type, { bubbles: true, cancelable: true, button: 0 }));
                }
            });
        });
        return () => {
            cancelAnimationFrame(f1);
            cancelAnimationFrame(f2);
        };
    }, []);
    return (
        <div className="max-w-md" ref={ref} style={{ height: 520 }}>
            {children}
        </div>
    );
}

function Field({ initial, groupNames = GROUPS }: { initial: string[]; groupNames?: string[] }) {
    const [selected, setSelected] = useState<string[]>(initial);
    return <PlanGroupsField groupNames={groupNames} selectedGroups={selected} onSelectedGroupsChange={setSelected} />;
}

/** No group picked: the placeholder. */
export const Empty = () => (
    <div className="max-w-md">
        <Field initial={[]} />
    </div>
);

/** Two groups picked, shown comma-joined in the closed input. */
export const TwoPicked = () => (
    <div className="max-w-md">
        <Field initial={["IS5 core", "Annihilation team"]} />
    </div>
);

/** Open: the create row, then every group with its tick and row actions. */
export const Open = () => (
    <OpenOnMount>
        <Field initial={["IS5 core", "Annihilation team"]} />
    </OpenOnMount>
);

/** Open with no groups yet: only the create row and "No groups found." */
export const OpenNoGroups = () => (
    <OpenOnMount>
        <Field initial={[]} groupNames={[]} />
    </OpenOnMount>
);
