import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { EyeOffIcon, PencilIcon, Trash2Icon } from "lucide-react";
import { useState } from "react";
import { Button } from "#/components/ui/button";
import { useErrorMessage } from "#/components/ui/error-message";
import { Kicker } from "#/components/ui/kicker";
import { Skeleton } from "#/components/ui/skeleton";
import { toastManager } from "#/components/ui/toast";
import { useAuth } from "#/hooks/use-auth";
import { deleteGridFn, type IGridSummary, myGridsQueryOptions } from "#/lib/api/grids";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { ConfirmDialog } from "./ConfirmDialog";
import { GridCard } from "./GridCard";
import type { messages } from "./MyGrids.messages";
import { NewGridButton } from "./NewGridButton";
import { DEFAULT_GRIDS_SEARCH, GRIDS_PER_USER_MAX } from "./shared";

export function MyGrids() {
    const t: TypedT<typeof messages> = useT("grids");
    const describeError = useErrorMessage();
    const queryClient = useQueryClient();
    const server = useGamedataServer();
    const { user } = useAuth();
    const options = myGridsQueryOptions(user?.id ?? null, server);
    const { data, isPending, isError, refetch, isFetching } = useQuery(options);
    const [deleting, setDeleting] = useState<IGridSummary | null>(null);
    const [deleteError, setDeleteError] = useState<string | null>(null);

    const remove = useMutation({
        mutationFn: (slug: string) => deleteGridFn({ data: slug }),
        onSuccess: (_, slug) => {
            const title = deleting?.title ?? "";
            queryClient.setQueryData<IGridSummary[]>(options.queryKey, (prev) => prev?.filter((g) => g.slug !== slug));
            void queryClient.invalidateQueries({ queryKey: ["grids"] });
            setDeleting(null);
            setDeleteError(null);
            toastManager.add({ id: `grid-delete-${Date.now()}`, title: t("my.toast.deleted", { title }), type: "success" });
        },
        onError: (err: unknown) => setDeleteError(describeError(err)),
    });

    const grids = data ?? [];

    return (
        <main className="min-h-dvh pb-24">
            <section className="page-gutter pt-10 pb-6 [--page-max:1080px] sm:pt-14 sm:pb-8">
                <div className="flex flex-wrap items-end justify-between gap-4">
                    <div className="min-w-0">
                        <Kicker>{t("my.kicker")}</Kicker>
                        <h1 className="m-0 font-bold font-sans text-3xl text-foreground leading-tight tracking-tight sm:text-4xl">{t("my.title")}</h1>
                        <p className="mt-2 font-sans text-muted-foreground text-sm">{t("my.count", { count: grids.length, max: GRIDS_PER_USER_MAX })}</p>
                    </div>
                    <div className="flex flex-wrap items-center gap-2">
                        <Button render={<Link to="/grids" search={DEFAULT_GRIDS_SEARCH} />} variant="outline">
                            {t("my.browse")}
                        </Button>
                        <NewGridButton />
                    </div>
                </div>
            </section>

            <div className="page-gutter [--page-max:1080px]">
                {isPending ? (
                    <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
                        {["s1", "s2", "s3"].map((k) => (
                            <Skeleton key={k} className="h-60 rounded-xl" />
                        ))}
                    </div>
                ) : isError ? (
                    <div className="rounded-lg border border-border border-dashed bg-muted/20 px-5 py-10 text-center">
                        <p className="m-0 font-sans text-muted-foreground text-sm">{t("my.error")}</p>
                        <Button type="button" variant="outline" size="sm" className="mt-3" onClick={() => void refetch()} loading={isFetching}>
                            {t("my.retry")}
                        </Button>
                    </div>
                ) : grids.length === 0 ? (
                    <div className="rounded-lg border border-border border-dashed bg-muted/20 px-5 py-12 text-center">
                        <p className="m-0 font-medium font-sans text-foreground text-sm">{t("my.emptyTitle")}</p>
                        <p className="mt-1 mb-4 font-sans text-[12.5px] text-muted-foreground">{t("my.emptyBody")}</p>
                        <NewGridButton />
                    </div>
                ) : (
                    <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
                        {grids.map((grid) => (
                            <GridCard
                                key={grid.slug}
                                grid={grid}
                                badge={
                                    grid.is_listed ? null : (
                                        <span className="inline-flex items-center gap-1 rounded-sm bg-muted px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground uppercase tracking-wider">
                                            <EyeOffIcon className="h-3 w-3" aria-hidden="true" />
                                            {t("my.unlisted")}
                                        </span>
                                    )
                                }
                                actions={
                                    <>
                                        <Button render={<Link to="/grids/$slug/edit" params={{ slug: grid.slug }} />} variant="outline" size="sm" className="flex-1">
                                            <PencilIcon />
                                            {t("my.edit")}
                                        </Button>
                                        <Button
                                            type="button"
                                            variant="ghost"
                                            size="icon-sm"
                                            aria-label={t("my.delete", { title: grid.title })}
                                            title={t("my.delete", { title: grid.title })}
                                            onClick={() => {
                                                setDeleteError(null);
                                                setDeleting(grid);
                                            }}
                                        >
                                            <Trash2Icon />
                                        </Button>
                                    </>
                                }
                            />
                        ))}
                    </div>
                )}
            </div>

            <ConfirmDialog
                open={deleting !== null}
                title={t("my.deleteDialog.title")}
                body={t("my.deleteDialog.body", { title: deleting?.title ?? "" })}
                confirmLabel={t("my.deleteDialog.confirm")}
                cancelLabel={t("my.deleteDialog.cancel")}
                destructive
                pending={remove.isPending}
                errorMessage={deleteError}
                onCancel={() => setDeleting(null)}
                onConfirm={() => deleting && remove.mutate(deleting.slug)}
            />
        </main>
    );
}
