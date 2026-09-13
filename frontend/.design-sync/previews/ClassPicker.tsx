import { ClassPicker } from "frontend";

const noop = () => {};

// The eight profession ids the picker iterates, in `CLASSES` order:
// WARRIOR SNIPER TANK MEDIC SUPPORT CASTER SPECIAL PIONEER. Each cell is a
// square button with the in-game profession icon; the tooltip gives the
// English class name (Guard, Defender, Vanguard, ...).
// The sidebar it lives in is 280px with 16px padding, so 248px of field width.
const Field = ({ children }: { children: React.ReactNode }) => <div className="w-64">{children}</div>;

export const NoneSelected = () => (
    <Field>
        <ClassPicker selected={[]} onChange={noop} />
    </Field>
);

/** Guard and Sniper toggled on - the primary-tinted pressed state. */
export const TwoSelected = () => (
    <Field>
        <ClassPicker selected={["WARRIOR", "SNIPER"]} onChange={noop} />
    </Field>
);

/** Everything but Specialist and Vanguard - a wide net, most cells pressed. */
export const MostSelected = () => (
    <Field>
        <ClassPicker selected={["WARRIOR", "SNIPER", "TANK", "MEDIC", "SUPPORT", "CASTER"]} onChange={noop} />
    </Field>
);
