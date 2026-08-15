import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle, Button } from "frontend";

export const OutlineClose = () => (
    <AlertDialog open>
        <AlertDialogPopup>
            <AlertDialogHeader>
                <AlertDialogTitle>Delete tier list?</AlertDialogTitle>
                <AlertDialogDescription>
                    <span className="font-medium text-foreground">Global Guard Rankings</span> and its 148 placements will be permanently removed. This cannot be undone.
                </AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
                <AlertDialogClose render={<Button type="button" variant="outline" />}>Cancel</AlertDialogClose>
                <Button type="button" variant="destructive">
                    Delete list
                </Button>
            </AlertDialogFooter>
        </AlertDialogPopup>
    </AlertDialog>
);

export const GhostClose = () => (
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

export const CloseIsTheOnlyAction = () => (
    <AlertDialog open>
        <AlertDialogPopup className="max-w-sm">
            <AlertDialogHeader>
                <AlertDialogTitle>Depot scan finished</AlertDialogTitle>
                <AlertDialogDescription>187 material counts were updated from your screenshot. 4 items could not be matched and were left unchanged.</AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
                <AlertDialogClose render={<Button type="button" className="sm:w-full" />}>Got it</AlertDialogClose>
            </AlertDialogFooter>
        </AlertDialogPopup>
    </AlertDialog>
);

export const CloseWhileSubmitting = () => (
    <AlertDialog open>
        <AlertDialogPopup>
            <AlertDialogHeader>
                <AlertDialogTitle>Unlink your EN account?</AlertDialogTitle>
                <AlertDialogDescription>Sync stops immediately. Nothing already imported is lost — relink at any time to resume.</AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
                <AlertDialogClose render={<Button type="button" variant="outline" />}>Cancel</AlertDialogClose>
                <Button type="button" variant="destructive" loading>
                    Unlink account
                </Button>
            </AlertDialogFooter>
        </AlertDialogPopup>
    </AlertDialog>
);
