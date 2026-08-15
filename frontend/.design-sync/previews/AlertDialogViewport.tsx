import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle, Button } from "frontend";

export const DefaultPlacement = () => (
    <AlertDialog open>
        <AlertDialogPopup>
            <AlertDialogHeader>
                <AlertDialogTitle>Stop syncing this account?</AlertDialogTitle>
                <AlertDialogDescription>The viewport anchors the dialog above centre so it clears the on-screen keyboard on mobile. Sync stops immediately; nothing already imported is lost.</AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
                <AlertDialogClose render={<Button type="button" variant="outline" />}>Cancel</AlertDialogClose>
                <Button type="button" variant="destructive">
                    Stop syncing
                </Button>
            </AlertDialogFooter>
        </AlertDialogPopup>
    </AlertDialog>
);

export const CompactDialog = () => (
    <AlertDialog open>
        <AlertDialogPopup className="max-w-sm">
            <AlertDialogHeader>
                <AlertDialogTitle>Discard draft?</AlertDialogTitle>
                <AlertDialogDescription>Your unsaved tier list draft will be lost.</AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
                <AlertDialogClose render={<Button type="button" variant="outline" />}>Keep</AlertDialogClose>
                <Button type="button" variant="destructive">
                    Discard
                </Button>
            </AlertDialogFooter>
        </AlertDialogPopup>
    </AlertDialog>
);

export const TallDialog = () => (
    <AlertDialog open>
        <AlertDialogPopup className="max-w-xl">
            <AlertDialogHeader>
                <AlertDialogTitle>Import 12 plans from your clipboard?</AlertDialogTitle>
                <AlertDialogDescription>Existing plans with the same operator are replaced. Review the list before confirming — this cannot be undone.</AlertDialogDescription>
            </AlertDialogHeader>
            <div className="px-6 pb-4">
                <ul className="divide-y rounded-lg border text-sm">
                    {[
                        ["Mlynar", "E2 90 · S3 M3"],
                        ["Skadi the Corrupting Heart", "E2 80 · S2 M3"],
                        ["Muelsyse", "E2 70 · Module X Stage 3"],
                        ["Texas the Omertosa", "E2 60 · S2 M3"],
                        ["Ines", "E2 60 · Module Y Stage 2"],
                    ].map(([name, goal]) => (
                        <li className="flex items-center justify-between px-3 py-2" key={name}>
                            <span className="font-medium">{name}</span>
                            <span className="font-mono text-muted-foreground text-xs tabular-nums">{goal}</span>
                        </li>
                    ))}
                </ul>
            </div>
            <AlertDialogFooter>
                <AlertDialogClose render={<Button type="button" variant="outline" />}>Cancel</AlertDialogClose>
                <Button type="button">Import 12 plans</Button>
            </AlertDialogFooter>
        </AlertDialogPopup>
    </AlertDialog>
);
