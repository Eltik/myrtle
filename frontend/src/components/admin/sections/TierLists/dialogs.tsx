import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { invalidateTierListGrants } from "#/components/admin/shell/invalidate";
import { toastError, toastSuccess } from "#/components/admin/shell/toast";
import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle } from "#/components/ui/alert-dialog";
import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { useErrorMessage } from "#/components/ui/error-message";
import { Field, FieldLabel } from "#/components/ui/field";
import { Input } from "#/components/ui/input";
import { Skeleton } from "#/components/ui/skeleton";
import { Textarea } from "#/components/ui/textarea";
import { useLastDefined } from "#/hooks/use-last-defined";
import { createTierListFn, deleteTierListFn, setTierListFlairFn, tierListFlairsQueryOptions } from "#/lib/api/tier-lists";
import { useT } from "#/lib/i18n";
import { cn } from "#/lib/utils";
import type { TierListsT } from "./labels";
import type { ITierListRow } from "./model";

export function NewListDialog({ open, onOpenChange }: { open: boolean; onOpenChange: (open: boolean) => void }): React.ReactElement {
    const t: TierListsT = useT("admin");
    const describeError = useErrorMessage();
    const queryClient = useQueryClient();
    const navigate = useNavigate();
    const [name, setName] = useState("");
    const [description, setDescription] = useState("");

    const create = useMutation({
        mutationFn: () => createTierListFn({ data: { name: name.trim(), description: description.trim() || null, listType: "official" } }),
        onSuccess: (list) => {
            void queryClient.invalidateQueries({ queryKey: ["tier-lists"] });
            toastSuccess("tier-lists-created", t("tierLists.new.toast"), t("tierLists.new.toastDesc"));
            onOpenChange(false);
            setName("");
            setDescription("");
            void navigate({ to: "/tier-lists/my/$id/edit", params: { id: list.slug } });
        },
        onError: (err: unknown) => toastError("tier-lists-create-failed", t("tierLists.new.failed"), describeError(err)),
    });

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogPopup className="max-w-md">
                <form
                    className="contents"
                    onSubmit={(e) => {
                        e.preventDefault();
                        if (name.trim() && !create.isPending) create.mutate();
                    }}
                >
                    <DialogHeader>
                        <DialogTitle>{t("tierLists.new.title")}</DialogTitle>
                        <DialogDescription>{t("tierLists.new.desc")}</DialogDescription>
                    </DialogHeader>
                    <DialogPanel className="flex flex-col gap-4">
                        <Field>
                            <FieldLabel>{t("tierLists.new.nameLabel")}</FieldLabel>
                            <Input value={name} onChange={(e) => setName(e.target.value)} placeholder={t("tierLists.new.namePlaceholder")} autoFocus />
                        </Field>
                        <Field>
                            <FieldLabel>{t("tierLists.new.descLabel")}</FieldLabel>
                            <Textarea value={description} onChange={(e) => setDescription(e.target.value)} placeholder={t("tierLists.new.descPlaceholder")} rows={3} />
                        </Field>
                    </DialogPanel>
                    <DialogFooter>
                        <DialogClose render={<Button variant="ghost" />}>{t("tierLists.cancel")}</DialogClose>
                        <Button type="submit" disabled={!name.trim()} loading={create.isPending}>
                            {t("tierLists.new.submit")}
                        </Button>
                    </DialogFooter>
                </form>
            </DialogPopup>
        </Dialog>
    );
}

export function FlairDialog({ list, onClose }: { list: ITierListRow | null; onClose: () => void }): React.ReactElement {
    const t: TierListsT = useT("admin");
    const describeError = useErrorMessage();
    const queryClient = useQueryClient();
    const shown = useLastDefined(list);
    const flairsQuery = useQuery({ ...tierListFlairsQueryOptions(), enabled: list !== null });
    const flairs = flairsQuery.data ?? [];
    const [selected, setSelected] = useState<number | null>(null);

    // Start from the list's current flair each time the dialog opens.
    const currentId = flairs.find((fl) => fl.code === list?.flairCode)?.id ?? null;
    useEffect(() => {
        if (list) setSelected(currentId);
    }, [list, currentId]);

    const save = useMutation({
        mutationFn: (input: { slug: string; flairId: number | null }) => setTierListFlairFn({ data: input }),
        onSuccess: () => {
            void queryClient.invalidateQueries({ queryKey: ["tier-lists"] });
            toastSuccess("tier-lists-flair", t("tierLists.flair.toast"), shown?.title);
            onClose();
        },
        onError: (err: unknown) => toastError("tier-lists-flair-failed", t("tierLists.flair.failed"), describeError(err)),
    });

    const chip = "inline-flex h-8 pointer-coarse:h-10 cursor-pointer items-center gap-1.5 rounded-md border border-border bg-card px-2.5 font-medium text-[13px] transition-shadow hover:bg-accent";

    return (
        <Dialog open={list !== null} onOpenChange={(open) => !open && onClose()}>
            <DialogPopup className="max-w-md">
                <DialogHeader>
                    <DialogTitle>{t("tierLists.flair.title")}</DialogTitle>
                    <DialogDescription>{t("tierLists.flair.desc", { title: shown?.title ?? "" })}</DialogDescription>
                </DialogHeader>
                <DialogPanel>
                    {flairsQuery.isPending ? (
                        <Skeleton className="h-8" />
                    ) : (
                        <div className="flex flex-wrap items-center gap-2">
                            <button type="button" aria-pressed={selected === null} onClick={() => setSelected(null)} className={cn(chip, selected === null && "ring-2 ring-ring")}>
                                <span className="size-2 rounded-full bg-muted-foreground/40" />
                                {t("tierLists.flair.none")}
                            </button>
                            {flairs.map((fl) => (
                                <button key={fl.code} type="button" aria-pressed={selected === fl.id} onClick={() => setSelected(fl.id)} className={cn(chip, selected === fl.id && "ring-2 ring-ring")} style={{ color: fl.color ?? undefined }}>
                                    <span className="size-2 rounded-full" style={{ background: fl.color ?? "var(--muted-foreground)" }} />
                                    {fl.label}
                                </button>
                            ))}
                        </div>
                    )}
                </DialogPanel>
                <DialogFooter>
                    <DialogClose render={<Button variant="ghost" />}>{t("tierLists.cancel")}</DialogClose>
                    <Button loading={save.isPending} disabled={!shown || selected === currentId} onClick={() => shown && save.mutate({ slug: shown.slug, flairId: selected })}>
                        {t("tierLists.save")}
                    </Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    );
}

export function DeleteListDialog({ list, onClose, onDeleted }: { list: ITierListRow | null; onClose: () => void; onDeleted: (slug: string) => void }): React.ReactElement {
    const t: TierListsT = useT("admin");
    const describeError = useErrorMessage();
    const queryClient = useQueryClient();
    const shown = useLastDefined(list);

    const del = useMutation({
        mutationFn: (slug: string) => deleteTierListFn({ data: slug }),
        onSuccess: (_res, slug) => {
            void queryClient.invalidateQueries({ queryKey: ["tier-lists"] });
            invalidateTierListGrants(queryClient);
            toastSuccess("tier-lists-deleted", t("tierLists.delete.toast"), shown?.title);
            onDeleted(slug);
            onClose();
        },
        onError: (err: unknown) => toastError("tier-lists-delete-failed", t("tierLists.delete.failed"), describeError(err)),
    });

    return (
        <AlertDialog open={list !== null} onOpenChange={(open) => !open && onClose()}>
            <AlertDialogPopup className="max-w-md">
                <AlertDialogHeader>
                    <AlertDialogTitle>{t("tierLists.delete.title", { title: shown?.title ?? "" })}</AlertDialogTitle>
                    <AlertDialogDescription>{t("tierLists.delete.body")}</AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                    <AlertDialogClose render={<Button variant="ghost" />}>{t("tierLists.cancel")}</AlertDialogClose>
                    <Button variant="destructive" loading={del.isPending} onClick={() => shown && del.mutate(shown.slug)}>
                        {t("tierLists.delete.submit")}
                    </Button>
                </AlertDialogFooter>
            </AlertDialogPopup>
        </AlertDialog>
    );
}
