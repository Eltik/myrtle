import { Link } from "@tanstack/react-router";
import { GitForkIcon } from "lucide-react";
import type { ReactNode } from "react";
import { FIT_CLASS } from "#/components/tier-lists/entities";
import { env } from "#/env";
import type { IGridSummary } from "#/lib/api/grids";
import { entityIconURL } from "#/lib/api/tier-entities";
import { useFormatters, useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { KindChips } from "./AllowedKinds";
import type { messages } from "./GridCard.messages";
import { gridSizeLabel } from "./shared";
import { thumbArtFit, thumbCells, thumbSpacing, thumbWidth } from "./thumb";

interface IGridCardProps {
    grid: IGridSummary;
    /** Next to the size, e.g. the unlisted badge on My grids. */
    badge?: ReactNode;
    /** Under the card's stats, e.g. edit and delete on My grids. */
    actions?: ReactNode;
}

export function GridCard({ grid, badge, actions }: IGridCardProps) {
    const t: TypedT<typeof messages> = useT("grids");
    const f = useFormatters();
    const forks = Number(grid.fork_count);

    return (
        <article className="group relative flex flex-col overflow-hidden rounded-xl border border-border bg-card shadow-[0_1px_2px_oklch(0_0_0/0.04)] transition-[border-color,box-shadow,transform] duration-200 hover:-translate-y-0.5 hover:border-foreground/20 hover:shadow-lg motion-reduce:transition-none motion-reduce:hover:translate-y-0">
            <Link to="/grids/$slug" params={{ slug: grid.slug }} className="flex flex-1 flex-col no-underline" aria-label={grid.title}>
                <GridThumb grid={grid} />
                <div className="flex flex-col gap-1.5 px-3.5 pt-3 pb-3">
                    <h3 className="m-0 line-clamp-1 font-sans font-semibold text-[15px] text-foreground leading-snug tracking-tight transition-colors group-hover:text-primary" title={grid.title}>
                        {grid.title}
                    </h3>
                    <div className="flex min-w-0 items-center gap-2 font-sans text-[11.5px] text-muted-foreground leading-none">
                        <span className="rounded-sm bg-muted px-1.5 py-0.5 font-mono text-[10.5px] text-foreground tabular-nums">{gridSizeLabel(grid.rows, grid.cols)}</span>
                        {badge}
                        <span className="min-w-0 truncate">{t("card.by", { name: grid.owner.name })}</span>
                        <span className="ml-auto inline-flex shrink-0 items-center gap-1 tabular-nums" title={t("card.forks", { count: forks, formatted: f.number(forks) })}>
                            <GitForkIcon className="h-3 w-3" aria-hidden="true" />
                            {f.compact(forks)}
                        </span>
                    </div>
                    <KindChips kinds={grid.entity_kinds} max={3} className="mt-0.5" />
                </div>
            </Link>
            {actions && <div className="flex items-center gap-1.5 border-border border-t px-3 py-2">{actions}</div>}
        </article>
    );
}

/** The board in miniature: every cell in the grid's shape, each pick's art where it has some, the board's empty grey elsewhere. No labels: at this size they would not read. */
function GridThumb({ grid }: { grid: IGridSummary }) {
    const server = useGamedataServer();
    const base = env.VITE_BACKEND_URL ?? "";
    const cells = thumbCells(grid.preview, grid.rows, grid.cols);
    const { gap, radius } = thumbSpacing(grid.rows, grid.cols);

    return (
        <div className="flex h-36 items-center justify-center bg-[oklch(0.13_0.004_285)] p-3 [container-type:size]">
            <div className="grid" style={{ gridTemplateColumns: `repeat(${grid.cols}, minmax(0, 1fr))`, gap, width: thumbWidth(grid.rows, grid.cols, gap) }} aria-hidden="true">
                {cells.map((cell, i) =>
                    cell ? (
                        // biome-ignore lint/suspicious/noArrayIndexKey: a cell IS its position
                        <span key={i} className="relative block aspect-square overflow-hidden bg-[oklch(0.24_0.005_285)]" style={{ borderRadius: radius }}>
                            <img
                                src={entityIconURL(cell.icon, base, cell.server ?? server)}
                                alt=""
                                loading="lazy"
                                decoding="async"
                                draggable={false}
                                onError={(e) => {
                                    e.currentTarget.style.visibility = "hidden";
                                }}
                                className={cn("absolute inset-0 block h-full w-full", FIT_CLASS[thumbArtFit(cell)])}
                            />
                        </span>
                    ) : (
                        // biome-ignore lint/suspicious/noArrayIndexKey: a cell IS its position
                        <span key={i} className="block aspect-square bg-[oklch(0.32_0.006_285)]" style={{ borderRadius: radius }} />
                    ),
                )}
            </div>
        </div>
    );
}
