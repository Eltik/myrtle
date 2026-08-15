import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle, Button, Card, CardDescription, CardHeader, CardPanel, CardTitle } from "frontend";

const PlannerBehind = () => (
    <div className="space-y-3 p-6">
        <h2 className="font-heading font-semibold text-lg">Upgrade planner</h2>
        <div className="space-y-2">
            {[
                { name: "Mlynar", goal: "E2 90 · S3 M3", left: "1.42M LMD" },
                { name: "Skadi the Corrupting Heart", goal: "E2 80 · S2 M3", left: "820K LMD" },
                { name: "Muelsyse", goal: "E2 70 · Module X", left: "4 Nanoflake" },
            ].map((p) => (
                <div className="flex items-center justify-between rounded-lg border px-3 py-2" key={p.name}>
                    <div className="min-w-0">
                        <p className="font-medium text-sm">{p.name}</p>
                        <p className="text-muted-foreground text-xs">{p.goal}</p>
                    </div>
                    <span className="font-mono text-muted-foreground text-xs tabular-nums">{p.left}</span>
                </div>
            ))}
        </div>
    </div>
);

export const OverlayOverPlanner = () => (
    <>
        <PlannerBehind />
        <AlertDialog open>
            <AlertDialogPopup>
                <AlertDialogHeader>
                    <AlertDialogTitle>Delete 3 plans?</AlertDialogTitle>
                    <AlertDialogDescription>
                        <span className="font-medium text-foreground">Mlynar, Skadi the Corrupting Heart, Muelsyse</span> and their promotion, level, skill, and module goals will be permanently removed.
                    </AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                    <AlertDialogClose render={<Button type="button" variant="outline" />}>Cancel</AlertDialogClose>
                    <Button type="button" variant="destructive">
                        Delete 3 plans
                    </Button>
                </AlertDialogFooter>
            </AlertDialogPopup>
        </AlertDialog>
    </>
);

export const OverlayOverCards = () => (
    <>
        <div className="grid gap-4 p-6 sm:grid-cols-2">
            <Card>
                <CardHeader>
                    <CardTitle>Sanity spent</CardTitle>
                    <CardDescription>Last 30 days</CardDescription>
                </CardHeader>
                <CardPanel className="font-semibold text-3xl tabular-nums">4,812</CardPanel>
            </Card>
            <Card>
                <CardHeader>
                    <CardTitle>Operators owned</CardTitle>
                    <CardDescription>Across all servers</CardDescription>
                </CardHeader>
                <CardPanel className="font-semibold text-3xl tabular-nums">231</CardPanel>
            </Card>
        </div>
        <AlertDialog open>
            <AlertDialogPopup>
                <AlertDialogHeader>
                    <AlertDialogTitle>Reset your statistics?</AlertDialogTitle>
                    <AlertDialogDescription>Your 30-day sanity history and clear counts go back to zero. Roster and depot data are untouched.</AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                    <AlertDialogClose render={<Button type="button" variant="outline" />}>Cancel</AlertDialogClose>
                    <Button type="button" variant="destructive">
                        Reset statistics
                    </Button>
                </AlertDialogFooter>
            </AlertDialogPopup>
        </AlertDialog>
    </>
);

export const NoOverlay = () => (
    <>
        <PlannerBehind />
        <AlertDialog>
            <AlertDialogPopup>
                <AlertDialogHeader>
                    <AlertDialogTitle>Delete 3 plans?</AlertDialogTitle>
                    <AlertDialogDescription>Closed — the overlay unmounts with the dialog and the planner reads at full contrast.</AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                    <AlertDialogClose render={<Button type="button" variant="outline" />}>Cancel</AlertDialogClose>
                    <Button type="button" variant="destructive">
                        Delete 3 plans
                    </Button>
                </AlertDialogFooter>
            </AlertDialogPopup>
        </AlertDialog>
    </>
);
