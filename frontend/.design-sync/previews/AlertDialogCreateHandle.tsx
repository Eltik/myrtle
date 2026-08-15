import { AlertDialog, AlertDialogClose, AlertDialogCreateHandle, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle, AlertDialogTrigger, Button, Card, CardDescription, CardHeader, CardPanel, CardTitle } from "frontend";
import { LinkIcon, Trash2Icon, TriangleAlertIcon } from "lucide-react";

// One handle per dialog, created once at module scope: any trigger anywhere in
// the tree can open it without being nested inside the AlertDialog root.
const unlinkAccount = AlertDialogCreateHandle();
const resetDepot = AlertDialogCreateHandle();

export const OpenedByHandle = () => (
    <>
        <div className="flex flex-wrap items-center gap-2 p-6">
            <AlertDialogTrigger handle={unlinkAccount} render={<Button variant="destructive-outline" />}>
                <LinkIcon />
                Unlink EN account
            </AlertDialogTrigger>
        </div>
        <AlertDialog handle={unlinkAccount} open>
            <AlertDialogPopup>
                <AlertDialogHeader>
                    <div className="flex items-center gap-3">
                        <span className="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-destructive/12 text-destructive-foreground">
                            <TriangleAlertIcon className="h-4.5 w-4.5" aria-hidden="true" />
                        </span>
                        <div className="flex min-w-0 flex-col gap-1">
                            <AlertDialogTitle>Unlink your EN account?</AlertDialogTitle>
                            <AlertDialogDescription>
                                Your roster of <span className="font-medium text-foreground">231 operators</span>, your depot, and 14 saved plans will stop syncing.
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
    </>
);

export const SharedByTwoTriggers = () => (
    <>
        <Card className="m-6 max-w-md">
            <CardHeader>
                <CardTitle>Depot</CardTitle>
                <CardDescription>187 material types tracked · last scanned 6 minutes ago</CardDescription>
            </CardHeader>
            <CardPanel className="flex flex-wrap gap-2">
                <AlertDialogTrigger handle={resetDepot} render={<Button size="sm" variant="outline" />}>
                    Reset counts
                </AlertDialogTrigger>
                <AlertDialogTrigger handle={resetDepot} render={<Button size="icon" variant="destructive-outline" aria-label="Reset depot" />}>
                    <Trash2Icon />
                </AlertDialogTrigger>
            </CardPanel>
        </Card>
        <AlertDialog handle={resetDepot}>
            <AlertDialogPopup>
                <AlertDialogHeader>
                    <AlertDialogTitle>Reset your depot?</AlertDialogTitle>
                    <AlertDialogDescription>All 187 manually edited material counts go back to zero.</AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                    <AlertDialogClose render={<Button type="button" variant="outline" />}>Cancel</AlertDialogClose>
                    <Button type="button" variant="destructive">
                        Reset depot
                    </Button>
                </AlertDialogFooter>
            </AlertDialogPopup>
        </AlertDialog>
    </>
);
