import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle, Button } from "frontend";

export const DefaultFooter = () => (
    <AlertDialog open>
        <AlertDialogPopup>
            <AlertDialogHeader>
                <AlertDialogTitle>Delete 3 plans?</AlertDialogTitle>
                <AlertDialogDescription>Mlynar, Skadi the Corrupting Heart, and Muelsyse lose their promotion, level, skill, and module goals. This cannot be undone.</AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
                <AlertDialogClose render={<Button type="button" variant="outline" />}>Cancel</AlertDialogClose>
                <Button type="button" variant="destructive">
                    Delete 3 plans
                </Button>
            </AlertDialogFooter>
        </AlertDialogPopup>
    </AlertDialog>
);

export const BareFooter = () => (
    <AlertDialog open>
        <AlertDialogPopup>
            <AlertDialogHeader>
                <AlertDialogTitle>Your sanity is about to overflow</AlertDialogTitle>
                <AlertDialogDescription>You are at 129 / 135 sanity. Spend it within the next 36 minutes or the surplus is lost.</AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter variant="bare">
                <AlertDialogClose render={<Button type="button" variant="ghost" />}>Remind me later</AlertDialogClose>
                <Button type="button">Open stage planner</Button>
            </AlertDialogFooter>
        </AlertDialogPopup>
    </AlertDialog>
);

export const SubmittingFooter = () => (
    <AlertDialog open>
        <AlertDialogPopup>
            <AlertDialogHeader>
                <AlertDialogTitle>Reset your depot?</AlertDialogTitle>
                <AlertDialogDescription>All 187 manually edited material counts go back to zero. Planner shortfalls will be recalculated against an empty depot.</AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
                <AlertDialogClose render={<Button type="button" variant="outline" />}>Cancel</AlertDialogClose>
                <Button type="button" variant="destructive" loading>
                    Reset depot
                </Button>
            </AlertDialogFooter>
        </AlertDialogPopup>
    </AlertDialog>
);

export const ThreeActions = () => (
    <AlertDialog open>
        <AlertDialogPopup className="max-w-xl">
            <AlertDialogHeader>
                <AlertDialogTitle>You have unsaved tier changes</AlertDialogTitle>
                <AlertDialogDescription>7 operators moved between tiers since your last save. Choose what to do before leaving the editor.</AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter className="sm:justify-between">
                <AlertDialogClose render={<Button type="button" variant="ghost" />}>Stay on page</AlertDialogClose>
                <div className="flex flex-col-reverse gap-2 sm:flex-row">
                    <AlertDialogClose render={<Button type="button" variant="destructive-outline" />}>Discard</AlertDialogClose>
                    <Button type="button">Save and leave</Button>
                </div>
            </AlertDialogFooter>
        </AlertDialogPopup>
    </AlertDialog>
);
