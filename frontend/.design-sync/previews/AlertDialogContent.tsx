import { AlertDialogClose, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle, AlertDialog as AlertDialogRoot, Button } from "frontend";
import { EraserIcon } from "lucide-react";

export const ResetDepot = () => (
    <AlertDialogRoot open>
        <AlertDialogContent>
            <AlertDialogHeader>
                <div className="flex items-center gap-3">
                    <span className="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-destructive/12 text-destructive-foreground">
                        <EraserIcon className="h-4.5 w-4.5" aria-hidden="true" />
                    </span>
                    <div className="flex min-w-0 flex-col gap-1">
                        <AlertDialogTitle>Reset your depot?</AlertDialogTitle>
                        <AlertDialogDescription>All 187 manually edited material counts go back to zero. Planner shortfalls will be recalculated against an empty depot.</AlertDialogDescription>
                    </div>
                </div>
            </AlertDialogHeader>
            <AlertDialogFooter>
                <AlertDialogClose render={<Button type="button" variant="outline" />}>Cancel</AlertDialogClose>
                <Button type="button" variant="destructive">
                    Reset depot
                </Button>
            </AlertDialogFooter>
        </AlertDialogContent>
    </AlertDialogRoot>
);

export const PublishList = () => (
    <AlertDialogRoot open>
        <AlertDialogContent>
            <AlertDialogHeader>
                <AlertDialogTitle>Publish “Global Guard Rankings”?</AlertDialogTitle>
                <AlertDialogDescription>Published lists appear in community search and accept votes from any signed-in doctor. You can unpublish later, but existing votes are kept.</AlertDialogDescription>
            </AlertDialogHeader>
            <div className="px-6 pb-4 text-sm">
                <dl className="space-y-1.5 rounded-lg border bg-muted/40 p-3">
                    <div className="flex justify-between">
                        <dt className="text-muted-foreground">Tiers</dt>
                        <dd className="font-mono tabular-nums">6</dd>
                    </div>
                    <div className="flex justify-between">
                        <dt className="text-muted-foreground">Operators placed</dt>
                        <dd className="font-mono tabular-nums">148</dd>
                    </div>
                    <div className="flex justify-between">
                        <dt className="text-muted-foreground">Unplaced</dt>
                        <dd className="font-mono tabular-nums">12</dd>
                    </div>
                </dl>
            </div>
            <AlertDialogFooter>
                <AlertDialogClose render={<Button type="button" variant="outline" />}>Not yet</AlertDialogClose>
                <Button type="button">Publish list</Button>
            </AlertDialogFooter>
        </AlertDialogContent>
    </AlertDialogRoot>
);

export const WideContent = () => (
    <AlertDialogRoot open>
        <AlertDialogContent className="max-w-xl">
            <AlertDialogHeader>
                <AlertDialogTitle>Recalculate the DPS table?</AlertDialogTitle>
                <AlertDialogDescription>The harness will re-run all 62 saved operator configurations against the current enemy set. This takes about 40 seconds and replaces the numbers currently shown.</AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
                <AlertDialogClose render={<Button type="button" variant="outline" />}>Cancel</AlertDialogClose>
                <Button type="button">Recalculate</Button>
            </AlertDialogFooter>
        </AlertDialogContent>
    </AlertDialogRoot>
);
