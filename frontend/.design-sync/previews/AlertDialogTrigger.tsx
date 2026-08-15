import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle, AlertDialogTrigger, Button, Card, CardDescription, CardHeader, CardPanel, CardTitle } from "frontend";
import { Trash2Icon, TriangleAlertIcon } from "lucide-react";

export const DangerZone = () => (
    <Card className="max-w-md">
        <CardHeader>
            <CardTitle>Danger zone</CardTitle>
            <CardDescription>These actions affect every device signed in to your account.</CardDescription>
        </CardHeader>
        <CardPanel className="flex flex-col gap-3">
            <div className="flex items-center justify-between gap-4">
                <div className="min-w-0">
                    <p className="font-medium text-sm">Clear planner history</p>
                    <p className="text-muted-foreground text-xs">Removes 14 saved plans and their material targets.</p>
                </div>
                <AlertDialog>
                    <AlertDialogTrigger render={<Button size="sm" variant="outline" />}>Clear</AlertDialogTrigger>
                    <AlertDialogPopup>
                        <AlertDialogHeader>
                            <AlertDialogTitle>Clear planner history?</AlertDialogTitle>
                            <AlertDialogDescription>All 14 saved plans will be removed. This cannot be undone.</AlertDialogDescription>
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
            <div className="flex items-center justify-between gap-4">
                <div className="min-w-0">
                    <p className="font-medium text-sm">Delete account</p>
                    <p className="text-muted-foreground text-xs">Your roster, depot, and tier lists are erased.</p>
                </div>
                <AlertDialog>
                    <AlertDialogTrigger render={<Button size="sm" variant="destructive-outline" />}>
                        <Trash2Icon />
                        Delete
                    </AlertDialogTrigger>
                    <AlertDialogPopup>
                        <AlertDialogHeader>
                            <AlertDialogTitle>Delete account?</AlertDialogTitle>
                            <AlertDialogDescription>This erases everything Myrtle stores for you and cannot be undone.</AlertDialogDescription>
                        </AlertDialogHeader>
                        <AlertDialogFooter>
                            <AlertDialogClose render={<Button type="button" variant="outline" />}>Cancel</AlertDialogClose>
                            <Button type="button" variant="destructive">
                                Delete account
                            </Button>
                        </AlertDialogFooter>
                    </AlertDialogPopup>
                </AlertDialog>
            </div>
        </CardPanel>
    </Card>
);

export const OpenedFromTrigger = () => (
    <AlertDialog defaultOpen>
        <AlertDialogTrigger render={<Button variant="destructive-outline" />}>
            <Trash2Icon />
            Delete tier list
        </AlertDialogTrigger>
        <AlertDialogPopup>
            <AlertDialogHeader>
                <div className="flex items-center gap-3">
                    <span className="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-destructive/12 text-destructive-foreground">
                        <TriangleAlertIcon className="h-4.5 w-4.5" aria-hidden="true" />
                    </span>
                    <div className="flex min-w-0 flex-col gap-1">
                        <AlertDialogTitle>Delete tier list?</AlertDialogTitle>
                        <AlertDialogDescription>
                            <span className="font-medium text-foreground">Sniper Rankings — CN meta</span> and its 96 placements will be permanently removed.
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

export const TriggerVariants = () => (
    <div className="flex flex-wrap items-center gap-2">
        <AlertDialog>
            <AlertDialogTrigger render={<Button variant="destructive" />}>Reset depot</AlertDialogTrigger>
        </AlertDialog>
        <AlertDialog>
            <AlertDialogTrigger render={<Button variant="outline" />}>Abandon run</AlertDialogTrigger>
        </AlertDialog>
        <AlertDialog>
            <AlertDialogTrigger render={<Button variant="ghost" />}>Sign out everywhere</AlertDialogTrigger>
        </AlertDialog>
        <AlertDialog>
            <AlertDialogTrigger render={<Button size="icon" variant="destructive-outline" aria-label="Delete plan" />}>
                <Trash2Icon />
            </AlertDialogTrigger>
        </AlertDialog>
    </div>
);
