import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle, Button, Card, CardDescription, CardHeader, CardPanel, CardTitle } from "frontend";

export const PortaledOverPage = () => (
    <>
        <div className="grid gap-4 p-6 sm:grid-cols-2">
            <Card>
                <CardHeader>
                    <CardTitle>Depot</CardTitle>
                    <CardDescription>187 material types tracked</CardDescription>
                </CardHeader>
                <CardPanel className="text-muted-foreground text-sm">Last scanned 6 minutes ago from your EN account.</CardPanel>
            </Card>
            <Card>
                <CardHeader>
                    <CardTitle>Planner</CardTitle>
                    <CardDescription>14 plans · 4 blocked on materials</CardDescription>
                </CardHeader>
                <CardPanel className="text-muted-foreground text-sm">3,120 sanity left to clear every goal.</CardPanel>
            </Card>
        </div>
        <AlertDialog open>
            <AlertDialogPopup>
                <AlertDialogHeader>
                    <AlertDialogTitle>Rescan your depot?</AlertDialogTitle>
                    <AlertDialogDescription>The portal lifts this dialog out of the card grid and onto the document body, so it is never clipped by an overflow-hidden ancestor.</AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                    <AlertDialogClose render={<Button type="button" variant="outline" />}>Cancel</AlertDialogClose>
                    <Button type="button">Rescan depot</Button>
                </AlertDialogFooter>
            </AlertDialogPopup>
        </AlertDialog>
    </>
);

export const KeptMounted = () => (
    <AlertDialog open>
        <AlertDialogPopup portalProps={{ keepMounted: true }}>
            <AlertDialogHeader>
                <AlertDialogTitle>Sign out of every device?</AlertDialogTitle>
                <AlertDialogDescription>You are signed in on 3 devices. Signing out everywhere ends those sessions immediately; your linked accounts stay linked.</AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
                <AlertDialogClose render={<Button type="button" variant="outline" />}>Cancel</AlertDialogClose>
                <Button type="button" variant="destructive">
                    Sign out everywhere
                </Button>
            </AlertDialogFooter>
        </AlertDialogPopup>
    </AlertDialog>
);

export const ClippingParent = () => (
    <div className="h-40 max-w-md overflow-hidden rounded-xl border p-4">
        <p className="font-medium text-sm">Recent clears</p>
        <p className="mt-1 text-muted-foreground text-xs">This panel clips its children, but the portalled dialog still renders at full size.</p>
        <AlertDialog open>
            <AlertDialogPopup>
                <AlertDialogHeader>
                    <AlertDialogTitle>Clear your run history?</AlertDialogTitle>
                    <AlertDialogDescription>All 1,284 recorded clears and their drop data will be removed from your account.</AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                    <AlertDialogClose render={<Button type="button" variant="outline" />}>Cancel</AlertDialogClose>
                    <Button type="button" variant="destructive">
                        Clear history
                    </Button>
                </AlertDialogFooter>
            </AlertDialogPopup>
        </AlertDialog>
    </div>
);
