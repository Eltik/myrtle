import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle, Button } from "frontend";

export const ShortTitle = () => (
    <AlertDialog open>
        <AlertDialogPopup>
            <AlertDialogHeader>
                <AlertDialogTitle>Discard draft?</AlertDialogTitle>
                <AlertDialogDescription>Your unsaved tier list draft will be lost.</AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
                <AlertDialogClose render={<Button type="button" variant="outline" />}>Keep editing</AlertDialogClose>
                <Button type="button" variant="destructive">
                    Discard
                </Button>
            </AlertDialogFooter>
        </AlertDialogPopup>
    </AlertDialog>
);

export const CountedTitle = () => (
    <AlertDialog open>
        <AlertDialogPopup>
            <AlertDialogHeader>
                <AlertDialogTitle>Remove 24 operators from this tier?</AlertDialogTitle>
                <AlertDialogDescription>They return to the unplaced pool at the bottom of the editor and keep their notes.</AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
                <AlertDialogClose render={<Button type="button" variant="outline" />}>Cancel</AlertDialogClose>
                <Button type="button" variant="destructive">
                    Remove 24
                </Button>
            </AlertDialogFooter>
        </AlertDialogPopup>
    </AlertDialog>
);

export const WrappingTitle = () => (
    <AlertDialog open>
        <AlertDialogPopup className="max-w-sm">
            <AlertDialogHeader>
                <AlertDialogTitle>Delete the Expeditioner's Joklumarkar run log?</AlertDialogTitle>
                <AlertDialogDescription>11 floors, 4 relics, and 240 banked candles are recorded in this log. Deleting it does not affect your account's candle balance.</AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
                <AlertDialogClose render={<Button type="button" variant="outline" />}>Cancel</AlertDialogClose>
                <Button type="button" variant="destructive">
                    Delete log
                </Button>
            </AlertDialogFooter>
        </AlertDialogPopup>
    </AlertDialog>
);
