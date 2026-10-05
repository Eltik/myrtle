import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Trash2Icon } from "lucide-react";
import { useState } from "react";
import { Button } from "#/components/ui/button";
import { useErrorMessage } from "#/components/ui/error-message";
import { toastManager } from "#/components/ui/toast";
import { deleteGridFn, type IGridSummary } from "#/lib/api/grids";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { ConfirmDialog } from "./ConfirmDialog";
import type { messages } from "./DeleteGrid.messages";

interface IDeleteGridButtonProps {
    grid: { slug: string; title: string };
    /** `icon`: a quiet trash button for cards. `labelled`: a destructive outline button reading "Delete". */
    variant?: "icon" | "labelled";
    /** `labelled` only: hide the word below the `sm` breakpoint, leaving the icon. */
    collapse?: boolean;
    disabled?: boolean;
    className?: string;
    /**
     * Runs after the DELETE succeeds and before the grid's cached detail is
     * dropped. A page showing the grid navigates away here: dropping the detail
     * while that page still observes it would refetch it and flash the
     * "not found" notice.
     */
    onDeleted?: () => void | Promise<void>;
}

/** The trigger, the confirmation and the mutation for deleting one grid. */
export function DeleteGridButton({ grid, variant = "icon", collapse = false, disabled = false, className, onDeleted }: IDeleteGridButtonProps) {
    const t: TypedT<typeof messages> = useT("grids");
    const describeError = useErrorMessage();
    const queryClient = useQueryClient();
    const [open, setOpen] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const remove = useMutation({
        mutationFn: (slug: string) => deleteGridFn({ data: slug }),
        onSuccess: async (_, slug) => {
            queryClient.setQueriesData<IGridSummary[]>({ queryKey: ["grids", "mine"] }, (prev) => prev?.filter((g) => g.slug !== slug));
            toastManager.add({ id: `grid-delete-${Date.now()}`, title: t("my.toast.deleted", { title: grid.title }), type: "success" });
            await onDeleted?.();
            setOpen(false);
            setError(null);
            queryClient.removeQueries({ queryKey: ["grids", "detail", slug] });
            void queryClient.invalidateQueries({ queryKey: ["grids"] });
        },
        onError: (err: unknown) => setError(describeError(err)),
    });

    const openDialog = () => {
        setError(null);
        setOpen(true);
    };

    const named = t("my.delete", { title: grid.title });

    return (
        <>
            {variant === "icon" ? (
                <Button type="button" variant="ghost" size="icon-sm" aria-label={named} title={named} disabled={disabled} className={className} onClick={openDialog}>
                    <Trash2Icon />
                </Button>
            ) : (
                <Button type="button" variant="destructive-outline" aria-label={collapse ? t("delete.action") : undefined} title={named} disabled={disabled} className={className} onClick={openDialog}>
                    <Trash2Icon />
                    <span className={cn(collapse && "hidden sm:inline")}>{t("delete.action")}</span>
                </Button>
            )}

            <ConfirmDialog
                open={open}
                title={t("my.deleteDialog.title")}
                body={t("my.deleteDialog.body", { title: grid.title })}
                confirmLabel={t("my.deleteDialog.confirm")}
                cancelLabel={t("my.deleteDialog.cancel")}
                destructive
                pending={remove.isPending}
                errorMessage={error}
                onCancel={() => setOpen(false)}
                onConfirm={() => remove.mutate(grid.slug)}
            />
        </>
    );
}
