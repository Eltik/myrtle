import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle, Button } from "frontend";

export const PlainDescription = () => (
    <AlertDialog open>
        <AlertDialogPopup>
            <AlertDialogHeader>
                <AlertDialogTitle>Stop tracking this event?</AlertDialogTitle>
                <AlertDialogDescription>Roaring Flare will disappear from your dashboard and its 4 farming reminders will be cancelled. Your clear history is kept.</AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
                <AlertDialogClose render={<Button type="button" variant="outline" />}>Keep tracking</AlertDialogClose>
                <Button type="button" variant="destructive">
                    Stop tracking
                </Button>
            </AlertDialogFooter>
        </AlertDialogPopup>
    </AlertDialog>
);

export const EmphasisedSubject = () => (
    <AlertDialog open>
        <AlertDialogPopup>
            <AlertDialogHeader>
                <AlertDialogTitle>Delete tier list?</AlertDialogTitle>
                <AlertDialogDescription>
                    <span className="font-medium text-foreground">Global Guard Rankings</span> and all of its tiers, placements, and stats will be permanently removed. Anyone with the link will see a 404 page. This cannot be undone.
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

export const MultiParagraph = () => (
    <AlertDialog open>
        <AlertDialogPopup className="max-w-xl">
            <AlertDialogHeader>
                <AlertDialogTitle>Import a roster from another account?</AlertDialogTitle>
                <AlertDialogDescription className="space-y-2">
                    <span className="block">Importing replaces your current roster of 231 operators with the 187 operators on the source account, including their levels, masteries, and modules.</span>
                    <span className="block">Your depot, tier lists, and 14 saved plans are not touched, but planner shortfalls will be recalculated against the new roster.</span>
                </AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
                <AlertDialogClose render={<Button type="button" variant="outline" />}>Cancel</AlertDialogClose>
                <Button type="button">Import roster</Button>
            </AlertDialogFooter>
        </AlertDialogPopup>
    </AlertDialog>
);
