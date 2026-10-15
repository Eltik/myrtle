import { ToggleField } from "frontend";
import { useState } from "react";

function Live({ id, label, initial }: { id: string; label: string; initial: boolean }) {
    const [on, setOn] = useState(initial);
    return <ToggleField id={id} label={label} checked={on} onChange={setOn} />;
}

// A labelled switch from the release planner's control rows.
export const Off = () => (
    <div className="p-4">
        <Live id="tf-stage-only" label="Stage events only" initial={false} />
    </div>
);

export const On = () => (
    <div className="p-4">
        <Live id="tf-auto" label="Auto-translate names" initial={true} />
    </div>
);

// In a control row, beside its siblings.
export const InARow = () => (
    <div className="flex flex-wrap items-center gap-x-4 gap-y-2 p-4">
        <Live id="tf-row-1" label="Stage events only" initial={true} />
        <Live id="tf-row-2" label="Auto-translate names" initial={true} />
        <Live id="tf-row-3" label="Show past rows" initial={false} />
    </div>
);
