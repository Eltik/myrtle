import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle, Button } from "frontend";
import { CloudOffIcon, TriangleAlertIcon } from "lucide-react";

export const PlainHeader = () => (
    <AlertDialog open>
        <AlertDialogPopup>
            <AlertDialogHeader>
                <AlertDialogTitle>Overwrite “Mlynar to M9”?</AlertDialogTitle>
                <AlertDialogDescription>A plan with that name was saved 3 days ago at E2 78. Importing this one replaces its goals and its 4 remaining material targets.</AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
                <AlertDialogClose render={<Button type="button" variant="outline" />}>Cancel</AlertDialogClose>
                <Button type="button">Overwrite</Button>
            </AlertDialogFooter>
        </AlertDialogPopup>
    </AlertDialog>
);

export const IconHeader = () => (
    <AlertDialog open>
        <AlertDialogPopup>
            <AlertDialogHeader>
                <div className="flex items-center gap-3">
                    <span className="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-destructive/12 text-destructive-foreground">
                        <TriangleAlertIcon className="h-4.5 w-4.5" aria-hidden="true" />
                    </span>
                    <div className="flex min-w-0 flex-col gap-1">
                        <AlertDialogTitle>Delete tier list?</AlertDialogTitle>
                        <AlertDialogDescription>
                            <span className="font-medium text-foreground">Global Guard Rankings</span> and all of its tiers, placements, and stats will be permanently removed. This cannot be undone.
                        </AlertDialogDescription>
                    </div>
                </div>
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

export const StackedIconHeader = () => (
    <AlertDialog open>
        <AlertDialogPopup className="max-w-sm">
            <AlertDialogHeader className="items-center text-center sm:text-center">
                <span className="inline-flex h-11 w-11 items-center justify-center rounded-full bg-muted text-muted-foreground">
                    <CloudOffIcon className="h-5 w-5" aria-hidden="true" />
                </span>
                <AlertDialogTitle>Sync is offline</AlertDialogTitle>
                <AlertDialogDescription>Myrtle can't reach the Yostar API. Your roster is showing the copy cached 4 hours ago.</AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter variant="bare">
                <AlertDialogClose render={<Button type="button" variant="outline" className="sm:flex-1" />}>Dismiss</AlertDialogClose>
                <Button type="button" className="sm:flex-1">
                    Retry sync
                </Button>
            </AlertDialogFooter>
        </AlertDialogPopup>
    </AlertDialog>
);
