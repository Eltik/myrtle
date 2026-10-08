import { RandomizerSwitchRow } from "frontend";
import { useState } from "react";

// A boolean rule in the randomizer's settings: label and description on the
// left, the switch on the right, the whole row a <label> so a click anywhere
// flips it. `locked` forces it off and disabled with a padlock when the rule
// needs data the session lacks (a linked profile, stage clears).

function Row({ label, description, initial, locked }: { label: string; description: string; initial: boolean; locked?: boolean }) {
    const [checked, setChecked] = useState(initial);
    return <RandomizerSwitchRow label={label} description={description} checked={checked} onChange={setChecked} locked={locked} />;
}

/** On. */
export const On = () => (
    <div className="max-w-sm">
        <Row label="Hide unplayable operators" description="Exclude tokens, support-only, and reserve operators." initial />
    </div>
);

/** Off. */
export const Off = () => (
    <div className="max-w-sm">
        <Row label="Allow duplicates" description="Same operator can appear twice in a squad." initial={false} />
    </div>
);

/** Locked: no linked profile, so the roster rule renders off, dimmed, with a padlock. */
export const Locked = () => (
    <div className="max-w-sm">
        <Row label="Only operators I own" description="Restrict to your roster from the linked profile." initial locked />
    </div>
);

/** The operator panel's "Rules" block for a signed-out visitor: two live rules, two locked. */
export const RulesBlock = () => (
    <div className="flex max-w-sm flex-col gap-2.5">
        <p className="font-mono text-[10.5px] text-muted-foreground/90 uppercase tracking-[0.18em]">Rules</p>
        <Row label="Allow duplicates" description="Same operator can appear twice in a squad." initial={false} />
        <Row label="Hide unplayable operators" description="Exclude tokens, support-only, and reserve operators." initial />
        <Row label="Only operators I own" description="Restrict to your roster from the linked profile." initial={false} locked />
        <Row label="E2 only" description="Restrict further to operators at elite 2 in your roster." initial={false} locked />
    </div>
);
