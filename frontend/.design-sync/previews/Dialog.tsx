import { Button, Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPopup, DialogTitle, Field, FieldDescription, FieldLabel, Input, Textarea } from "frontend";
import { TriangleAlertIcon } from "lucide-react";
import type { ReactNode } from "react";

// The popup portals into a `fixed inset-0` viewport, so every open story needs a
// stage tall enough for the backdrop to fill. Interaction (open/close, focus
// trap, dismiss) cannot be photographed — these are the resting open states.
const Stage = ({ children }: { children?: ReactNode }) => <div className="relative min-h-[520px] w-full">{children}</div>;

export const NewTierList = () => (
    <Stage>
        <Dialog open>
            <DialogPopup>
                <DialogHeader>
                    <DialogTitle>New tier list</DialogTitle>
                    <DialogDescription>Give your list a name and a short description. You can change these later, and your edits are visible to everyone with the share link.</DialogDescription>
                </DialogHeader>
                <div className="flex flex-col gap-5 px-6 pb-2">
                    <Field>
                        <FieldLabel className="w-full" htmlFor="tier-list-name">
                            Name
                            <span className="ml-auto font-mono text-[10.5px] text-muted-foreground tabular-nums">20 / 64</span>
                        </FieldLabel>
                        <Input defaultValue="Endgame DPS rankings" id="tier-list-name" />
                        <FieldDescription>Shown on browse cards and on the public detail page.</FieldDescription>
                    </Field>
                    <Field>
                        <FieldLabel className="w-full" htmlFor="tier-list-description">
                            Description
                            <span className="ml-auto font-mono text-[10.5px] text-muted-foreground tabular-nums">78 / 280</span>
                        </FieldLabel>
                        <Textarea defaultValue="Ranked for CC risk 18 and above — assumes M3 skills, no Module gating." id="tier-list-description" rows={3} />
                        <FieldDescription>Optional. A sentence or two helps readers know what to expect.</FieldDescription>
                    </Field>
                </div>
                <DialogFooter>
                    <DialogClose render={<Button type="button" variant="outline" />}>Cancel</DialogClose>
                    <Button type="submit">Create list</Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </Stage>
);

export const ConfirmDelete = () => (
    <Stage>
        <Dialog open>
            <DialogPopup className="max-w-md">
                <DialogHeader>
                    <div className="flex items-start gap-3">
                        <span className="inline-flex size-9 shrink-0 items-center justify-center rounded-full bg-destructive/12 text-destructive-foreground">
                            <TriangleAlertIcon aria-hidden="true" className="size-4.5" />
                        </span>
                        <div className="flex min-w-0 flex-col gap-1">
                            <DialogTitle>Delete tier list?</DialogTitle>
                            <DialogDescription>
                                <span className="font-medium text-foreground">CC#14 must-haves</span> and all of its tiers, placements, and stats will be permanently removed. Anyone with the link will see a 404. This cannot be undone.
                            </DialogDescription>
                        </div>
                    </div>
                </DialogHeader>
                <DialogFooter>
                    <DialogClose render={<Button type="button" variant="outline" />}>Cancel</DialogClose>
                    <Button type="button" variant="destructive">
                        Delete list
                    </Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </Stage>
);

export const SignIn = () => (
    <Stage>
        <Dialog open>
            <DialogPopup className="max-w-sm">
                <DialogHeader>
                    <DialogTitle>Sign in to Myrtle</DialogTitle>
                    <DialogDescription>Sync your depot, planner goals and tier lists across devices.</DialogDescription>
                </DialogHeader>
                <div className="flex flex-col gap-4 px-6 pb-2">
                    <Field>
                        <FieldLabel htmlFor="sign-in-email">Email</FieldLabel>
                        <Input defaultValue="doctor@rhodes.island" id="sign-in-email" type="email" />
                    </Field>
                    <Field>
                        <FieldLabel htmlFor="sign-in-password">Password</FieldLabel>
                        <Input defaultValue="••••••••••" id="sign-in-password" type="password" />
                        <FieldDescription>Eight characters minimum.</FieldDescription>
                    </Field>
                </div>
                <DialogFooter>
                    <DialogClose render={<Button type="button" variant="ghost" />}>Not now</DialogClose>
                    <Button type="submit">Sign in</Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </Stage>
);
