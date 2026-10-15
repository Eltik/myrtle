import { CheckMark } from "frontend";

// The tick box at the start of a multi-select combobox row. The operator and
// group pickers in the planner draw it themselves (it is not a Checkbox: the
// whole row is the control, so the box is decoration only). Filled brand red
// when ticked, an empty input-bordered square when not.

/** Ticked: the row is part of the selection. */
export const Checked = () => <CheckMark checked />;

/** Unticked: an empty square in the input border colour. */
export const Unchecked = () => <CheckMark checked={false} />;

const groups = [
    { name: "Annihilation team", checked: true },
    { name: "IS5 core", checked: true },
    { name: "Contingency Contract", checked: false },
    { name: "Base shift 2", checked: false },
];

/** As the group picker lays it out: the tick leads each row, the label follows. */
export const InPickerRows = () => (
    <div className="w-72 rounded-lg border border-border bg-popover p-1 text-popover-foreground shadow-lg">
        {groups.map((group) => (
            <div key={group.name} className="flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm">
                <CheckMark checked={group.checked} />
                <span className="truncate">{group.name}</span>
            </div>
        ))}
    </div>
);
