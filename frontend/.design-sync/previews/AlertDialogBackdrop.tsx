import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle, Badge, Button, Card, CardDescription, CardHeader, CardPanel, CardTitle } from "frontend";

const ListPageBehind = () => (
    <div className="grid gap-4 p-6 sm:grid-cols-2">
        <Card>
            <CardHeader>
                <CardTitle>Global Guard Rankings</CardTitle>
                <CardDescription>148 operators placed across 6 tiers</CardDescription>
            </CardHeader>
            <CardPanel className="text-muted-foreground text-sm">Updated 3 days ago · 312 votes</CardPanel>
        </Card>
        <Card>
            <CardHeader>
                <CardTitle>Sniper Rankings — CN meta</CardTitle>
                <CardDescription>96 operators placed across 5 tiers</CardDescription>
            </CardHeader>
            <CardPanel className="text-muted-foreground text-sm">Updated 2 weeks ago · 58 votes</CardPanel>
        </Card>
    </div>
);

export const OverPageContent = () => (
    <>
        <ListPageBehind />
        <AlertDialog open>
            <AlertDialogPopup>
                <AlertDialogHeader>
                    <AlertDialogTitle>Delete tier list?</AlertDialogTitle>
                    <AlertDialogDescription>
                        <span className="font-medium text-foreground">Sniper Rankings — CN meta</span> and its 96 placements will be permanently removed. Anyone with the link will see a 404 page.
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
    </>
);

export const OverDenseContent = () => (
    <>
        <div className="space-y-4 p-6">
            <div className="flex flex-wrap gap-2">
                <Badge>Chapter 8</Badge>
                <Badge variant="secondary">Roaring Flare</Badge>
                <Badge variant="secondary">14 stages</Badge>
                <Badge variant="secondary">3 challenge modes</Badge>
            </div>
            <table className="w-full text-sm">
                <thead className="text-muted-foreground text-xs">
                    <tr className="border-b">
                        <th className="py-2 text-left font-medium">Stage</th>
                        <th className="py-2 text-left font-medium">Sanity</th>
                        <th className="py-2 text-right font-medium">Efficiency</th>
                    </tr>
                </thead>
                <tbody className="font-mono tabular-nums">
                    <tr className="border-b">
                        <td className="py-1.5 font-sans">1-7</td>
                        <td className="py-1.5">6</td>
                        <td className="py-1.5 text-right">0.72</td>
                    </tr>
                    <tr className="border-b">
                        <td className="py-1.5 font-sans">S4-1</td>
                        <td className="py-1.5">18</td>
                        <td className="py-1.5 text-right">0.68</td>
                    </tr>
                    <tr>
                        <td className="py-1.5 font-sans">CE-6</td>
                        <td className="py-1.5">36</td>
                        <td className="py-1.5 text-right">0.94</td>
                    </tr>
                </tbody>
            </table>
        </div>
        <AlertDialog open>
            <AlertDialogPopup>
                <AlertDialogHeader>
                    <AlertDialogTitle>Leave without saving?</AlertDialogTitle>
                    <AlertDialogDescription>You moved 7 operators between tiers since your last save. Leaving now discards those changes.</AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                    <AlertDialogClose render={<Button type="button" variant="outline" />}>Stay on page</AlertDialogClose>
                    <Button type="button" variant="destructive">
                        Discard changes
                    </Button>
                </AlertDialogFooter>
            </AlertDialogPopup>
        </AlertDialog>
    </>
);

export const Closed = () => (
    <>
        <ListPageBehind />
        <AlertDialog>
            <AlertDialogPopup>
                <AlertDialogHeader>
                    <AlertDialogTitle>Delete tier list?</AlertDialogTitle>
                    <AlertDialogDescription>The backdrop unmounts with the dialog, so the page underneath is fully interactive again.</AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                    <AlertDialogClose render={<Button type="button" variant="outline" />}>Cancel</AlertDialogClose>
                    <Button type="button" variant="destructive">
                        Delete list
                    </Button>
                </AlertDialogFooter>
            </AlertDialogPopup>
        </AlertDialog>
    </>
);
