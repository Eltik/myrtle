import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle, Button } from "frontend";
import { TriangleAlertIcon } from "lucide-react";

export const DeleteAccount = () => (
    <AlertDialog open>
        <AlertDialogPopup>
            <AlertDialogHeader>
                <div className="flex items-center gap-3">
                    <span className="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-destructive/12 text-destructive-foreground">
                        <TriangleAlertIcon className="h-4.5 w-4.5" aria-hidden="true" />
                    </span>
                    <div className="flex min-w-0 flex-col gap-1">
                        <AlertDialogTitle>Unlink your EN account?</AlertDialogTitle>
                        <AlertDialogDescription>
                            Your roster of <span className="font-medium text-foreground">231 operators</span>, your depot, and 14 saved plans will stop syncing. Nothing is deleted — relink at any time to restore them.
                        </AlertDialogDescription>
                    </div>
                </div>
            </AlertDialogHeader>
            <AlertDialogFooter>
                <AlertDialogClose render={<Button type="button" variant="outline" />}>Keep linked</AlertDialogClose>
                <Button type="button" variant="destructive">
                    Unlink account
                </Button>
            </AlertDialogFooter>
        </AlertDialogPopup>
    </AlertDialog>
);

export const OverwritePlan = () => (
    <AlertDialog open>
        <AlertDialogPopup>
            <AlertDialogHeader>
                <AlertDialogTitle>Overwrite “Mlynar to M9”?</AlertDialogTitle>
                <AlertDialogDescription>A plan with that name already exists, saved 3 days ago at E2 78. Importing this one replaces its goals and its 4 remaining material targets.</AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
                <AlertDialogClose render={<Button type="button" variant="outline" />}>Cancel</AlertDialogClose>
                <Button type="button">Overwrite plan</Button>
            </AlertDialogFooter>
        </AlertDialogPopup>
    </AlertDialog>
);

export const AbandonRun = () => (
    <AlertDialog open>
        <AlertDialogPopup>
            <AlertDialogHeader>
                <AlertDialogTitle>Abandon this Integrated Strategies run?</AlertDialogTitle>
                <AlertDialogDescription>You are 11 floors into Expeditioner's Joklumarkar with 4 relics collected. Abandoning forfeits the run's remaining rewards and the 240 candles banked this week.</AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
                <AlertDialogClose render={<Button type="button" variant="outline" />}>Keep playing</AlertDialogClose>
                <Button type="button" variant="destructive">
                    Abandon run
                </Button>
            </AlertDialogFooter>
        </AlertDialogPopup>
    </AlertDialog>
);
