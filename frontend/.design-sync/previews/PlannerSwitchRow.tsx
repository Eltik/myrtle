import { PlannerSwitchRow } from "frontend";
import { useState } from "react";

// The planner's labelled on/off setting: its own bordered card, label and
// description on the left, the switch on the right. The plan dialog uses one
// for "Display on profile"; the bulk-add dialog stacks two.

function Row({ id, label, description, initial }: { id: string; label: string; description: string; initial: boolean }) {
    const [checked, setChecked] = useState(initial);
    return <PlannerSwitchRow id={id} label={label} description={description} checked={checked} onCheckedChange={setChecked} />;
}

/** On: the plan shows on the player's public profile. */
export const On = () => (
    <div className="max-w-md">
        <Row id="display-on-profile" label="Display on profile" description="Show this target plan on your public user profile." initial />
    </div>
);

/** Off: the default for a new plan. */
export const Off = () => (
    <div className="max-w-md">
        <Row id="display-on-profile-off" label="Display on profile" description="Show this target plan on your public user profile." initial={false} />
    </div>
);

/** The bulk-add dialog's pair, stacked as it renders them. */
export const BulkDialogPair = () => {
    const [shown, setShown] = useState(true);
    const [overwrite, setOverwrite] = useState(false);
    return (
        <div className="max-w-md space-y-3">
            <PlannerSwitchRow className="gap-4" id="bulk-display-on-profile" label="Display on profile" description="Show these plans on your public profile. Plans already shown there stay shown." checked={shown} onCheckedChange={setShown} />
            <PlannerSwitchRow className="gap-4" id="bulk-overwrite" label="Replace existing plans" description="When off, operators that already have a plan are hidden from the picker." checked={overwrite} onCheckedChange={setOverwrite} />
        </div>
    );
};
