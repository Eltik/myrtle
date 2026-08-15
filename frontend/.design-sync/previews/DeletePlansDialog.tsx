import { DeletePlansDialog } from "frontend";

// Destructive confirmation for the planner. `target` doubles as the open flag:
// a non-null target opens the dialog and drives all of the copy (title, name
// list, singular/plural button label).

const noop = () => {};

export const SinglePlan = () => <DeletePlansDialog errorMessage={null} isSubmitting={false} onConfirm={noop} onOpenChange={noop} target={{ ids: ["char_4064_mlynar"], names: ["Młynar"] }} />;

export const ThreePlans = () => <DeletePlansDialog errorMessage={null} isSubmitting={false} onConfirm={noop} onOpenChange={noop} target={{ ids: ["char_4064_mlynar", "char_263_skadi", "char_180_amgoat"], names: ["Młynar", "Skadi", "Eyjafjalla"] }} />;

// Past three names the list truncates to "… and N more".
export const ManyPlans = () => (
    <DeletePlansDialog
        errorMessage={null}
        isSubmitting={false}
        onConfirm={noop}
        onOpenChange={noop}
        target={{
            ids: ["char_4064_mlynar", "char_263_skadi", "char_180_amgoat", "char_102_texas", "char_202_demkni", "char_128_plosis", "char_151_myrtle"],
            names: ["Młynar", "Skadi", "Eyjafjalla", "Texas", "Saria", "Ptilopsis", "Myrtle"],
        }}
    />
);

// The delete call failed: the inline alert appears above the footer and both
// actions stay live so the doctor can retry or back out.
export const DeleteFailed = () => <DeletePlansDialog errorMessage="Failed to delete. Please try again." isSubmitting={false} onConfirm={noop} onOpenChange={noop} target={{ ids: ["char_1028_texas2", "char_249_mlyss"], names: ["Texas the Omertosa", "Muelsyse"] }} />;
