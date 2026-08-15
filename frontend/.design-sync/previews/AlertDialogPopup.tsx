import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle, Button } from "frontend";
import { TriangleAlertIcon } from "lucide-react";

export const DestructiveConfirm = () => (
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
                            <span className="font-medium text-foreground">Global Guard Rankings</span> and all of its tiers, placements, and stats will be permanently removed. Anyone with the link will see a 404 page. This cannot be undone.
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

export const WithErrorMessage = () => (
    <AlertDialog open>
        <AlertDialogPopup>
            <AlertDialogHeader>
                <div className="flex items-center gap-3">
                    <span className="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-destructive/12 text-destructive-foreground">
                        <TriangleAlertIcon className="h-4.5 w-4.5" aria-hidden="true" />
                    </span>
                    <div className="flex min-w-0 flex-col gap-1">
                        <AlertDialogTitle>Delete 3 plans?</AlertDialogTitle>
                        <AlertDialogDescription>
                            <span className="font-medium text-foreground">Mlynar E2 90, Skadi the Corrupting Heart M3, Muelsyse module X</span> and their promotion, level, skill, and module goals will be permanently removed. This cannot be undone.
                        </AlertDialogDescription>
                    </div>
                </div>
            </AlertDialogHeader>
            <div role="alert" className="mx-6 mb-2 rounded-lg border border-destructive/30 bg-destructive/8 px-3 py-2 font-sans text-destructive-foreground text-xs">
                Couldn't reach the planner service. Your plans were not deleted — try again in a moment.
            </div>
            <AlertDialogFooter>
                <AlertDialogClose render={<Button type="button" variant="outline" />}>Cancel</AlertDialogClose>
                <Button type="button" variant="destructive" loading>
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
