import { useMutation, useQueryClient, useSuspenseQuery } from "@tanstack/react-query";
import { Link, useNavigate } from "@tanstack/react-router";
import { ArrowLeftIcon, CopyIcon, Download, GitForkIcon, PencilIcon } from "lucide-react";
import { useMemo, useState } from "react";
import { Button } from "#/components/ui/button";
import { useErrorMessage } from "#/components/ui/error-message";
import { toastManager } from "#/components/ui/toast";
import { useAuth } from "#/hooks/use-auth";
import { forkGridFn, gridQueryOptions, type IGrid } from "#/lib/api/grids";
import { authActions } from "#/lib/auth/store";
import { useFormatters, useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { gridImageFilename } from "#/lib/og/impl/grid";
import { cn, downloadBlob } from "#/lib/utils";
import { KindChips } from "./AllowedKinds";
import { DeleteGridButton } from "./DeleteGrid";
import { GridBoard } from "./GridBoard";
import { GridNotice } from "./GridNotice";
import type { messages } from "./GridView.messages";
import { DEFAULT_GRIDS_SEARCH, gridSizeLabel } from "./shared";
import { gridToState } from "./state";

export function GridView({ slug }: { slug: string }) {
    const t: TypedT<typeof messages> = useT("grids");
    const server = useGamedataServer();
    const { user } = useAuth();
    const { data: grid } = useSuspenseQuery(gridQueryOptions(slug, user?.id ?? null, server));

    if (!grid) {
        return (
            <GridNotice
                title={t("view.missing.title")}
                body={t("view.missing.body")}
                action={
                    <Button render={<Link to="/grids" search={DEFAULT_GRIDS_SEARCH} />} variant="outline" className="mt-6">
                        {t("view.missing.action")}
                    </Button>
                }
            />
        );
    }
    return <GridViewContent grid={grid} />;
}

function GridViewContent({ grid }: { grid: IGrid }) {
    const t: TypedT<typeof messages> = useT("grids");
    const f = useFormatters();
    const state = useMemo(() => gridToState(grid), [grid]);

    return (
        <main className="min-h-dvh pb-24">
            <div className="page-gutter pt-6 [--page-max:1180px] sm:pt-10">
                <Link to="/grids" search={DEFAULT_GRIDS_SEARCH} className="inline-flex items-center gap-1.5 font-sans text-muted-foreground text-xs no-underline hover:text-foreground">
                    <ArrowLeftIcon className="h-3.5 w-3.5" aria-hidden="true" />
                    {t("view.back")}
                </Link>

                <div className="mt-4 flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
                    <div className="flex min-w-0 flex-col gap-1.5">
                        <div className="flex flex-wrap items-center gap-x-2 gap-y-1 font-sans text-muted-foreground text-xs">
                            <span>{t("view.by", { name: grid.owner.name })}</span>
                            <span aria-hidden="true">·</span>
                            <span className="font-mono tabular-nums">{gridSizeLabel(grid.rows, grid.cols)}</span>
                            <span aria-hidden="true">·</span>
                            <span className="inline-flex items-center gap-1">
                                <GitForkIcon className="h-3.5 w-3.5" aria-hidden="true" />
                                {t("view.forks", { count: Number(grid.fork_count), formatted: f.number(Number(grid.fork_count)) })}
                            </span>
                            {!grid.is_listed && (
                                <>
                                    <span aria-hidden="true">·</span>
                                    <span>{t("view.unlisted")}</span>
                                </>
                            )}
                        </div>
                        <KindChips kinds={grid.entity_kinds} />
                        {grid.template_of && (
                            <p className="m-0 font-sans text-muted-foreground text-xs">
                                {t("view.templateOf")}{" "}
                                <Link to="/grids/$slug" params={{ slug: grid.template_of.slug }} className="font-medium text-foreground underline-offset-2 hover:underline">
                                    {grid.template_of.title}
                                </Link>
                            </p>
                        )}
                    </div>
                    <GridActions grid={grid} />
                </div>

                <div className="mt-5">
                    <GridBoard title={state.title} rows={state.rows} cols={state.cols} cells={state.cells} />
                </div>

                {grid.description && <p className="mx-auto mt-5 max-w-180 whitespace-pre-line text-center font-sans text-muted-foreground text-sm leading-relaxed">{grid.description}</p>}
            </div>
        </main>
    );
}

const ACTION_CLASS = "inline-flex h-11 min-w-11 cursor-pointer items-center justify-center gap-1.5 rounded-lg border border-border bg-popover px-3 font-medium font-sans text-foreground text-sm leading-none no-underline transition-colors hover:bg-accent disabled:cursor-wait disabled:opacity-60 sm:h-9 sm:min-w-0";

function GridActions({ grid }: { grid: IGrid }) {
    const t: TypedT<typeof messages> = useT("grids");
    const describeError = useErrorMessage();
    const queryClient = useQueryClient();
    const navigate = useNavigate();
    const server = useGamedataServer();
    const { user } = useAuth();
    const [downloading, setDownloading] = useState(false);

    const fork = useMutation({
        mutationFn: () => forkGridFn({ data: { slug: grid.slug, server } }),
        onSuccess: (created) => {
            void queryClient.invalidateQueries({ queryKey: ["grids"] });
            navigate({ to: "/grids/$slug/edit", params: { slug: created.slug } });
        },
        onError: (err: unknown) => {
            toastManager.add({ id: `grid-fork-err-${Date.now()}`, title: t("view.fork.errTitle"), description: describeError(err), type: "error" });
        },
    });

    const handleFork = () => {
        if (!user) {
            authActions.openLoginDialog(typeof window === "undefined" ? null : window.location.pathname);
            return;
        }
        if (!fork.isPending) fork.mutate();
    };

    const handleDownload = async () => {
        if (downloading) return;
        setDownloading(true);
        try {
            const res = await fetch(`/api/grids/${encodeURIComponent(grid.slug)}/image?server=${encodeURIComponent(server)}&v=${encodeURIComponent(`${grid.updated_at}:${server}`)}`);
            if (!res.ok) throw new Error(`HTTP ${res.status}`);
            const blob = await res.blob();
            downloadBlob(blob, gridImageFilename(grid.slug));
        } catch {
            toastManager.add({ id: `grid-download-err-${Date.now()}`, title: t("view.download.errTitle"), description: t("view.download.errBody"), type: "error" });
        } finally {
            setDownloading(false);
        }
    };

    const handleCopy = async () => {
        if (typeof window === "undefined") return;
        try {
            await navigator.clipboard.writeText(`${window.location.origin}/grids/${grid.slug}`);
            toastManager.add({ id: `grid-copy-${Date.now()}`, title: t("view.copy.okTitle"), type: "success" });
        } catch {
            toastManager.add({ id: `grid-copy-err-${Date.now()}`, title: t("view.copy.errTitle"), type: "error" });
        }
    };

    return (
        <div className="flex shrink-0 flex-wrap items-center gap-2">
            <Button type="button" onClick={handleFork} loading={fork.isPending} className="h-11 sm:h-9">
                <GitForkIcon />
                {t("view.fork")}
            </Button>
            {grid.can_edit && (
                <Link to="/grids/$slug/edit" params={{ slug: grid.slug }} className={ACTION_CLASS} aria-label={t("view.edit")}>
                    <PencilIcon className="h-4 w-4" aria-hidden="true" />
                    <span className="hidden sm:inline">{t("view.edit")}</span>
                </Link>
            )}
            {grid.can_edit && <DeleteGridButton grid={grid} variant="labelled" collapse className="h-11 min-w-11 rounded-lg px-3 sm:h-9 sm:min-w-0" onDeleted={() => navigate({ to: "/grids/my", replace: true })} />}
            <button type="button" onClick={handleDownload} disabled={downloading} className={ACTION_CLASS} aria-label={t("view.download")}>
                <Download className={cn("h-4 w-4", downloading && "animate-pulse")} aria-hidden="true" />
                <span className="hidden sm:inline">{downloading ? t("view.downloading") : t("view.download")}</span>
            </button>
            <button type="button" onClick={handleCopy} className={ACTION_CLASS} aria-label={t("view.copy")}>
                <CopyIcon className="h-4 w-4" aria-hidden="true" />
                <span className="hidden sm:inline">{t("view.copy")}</span>
            </button>
        </div>
    );
}
