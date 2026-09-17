import { ChevronRight } from "lucide-react";
import type * as React from "react";
import { Fragment } from "react";
import { cn } from "#/lib/utils";

export interface IPageHeaderProps {
    /** Crumbs above the title, outermost first; the last one is the current page and renders in the foreground colour. Omit for a page with no trail. */
    breadcrumb?: readonly React.ReactNode[];
    /** Accessible name of the breadcrumb nav. Required whenever `breadcrumb` is given. */
    breadcrumbLabel?: string;
    title: React.ReactNode;
    /** Present selects the WITH-DESCRIPTION design; absent selects the base design. There is no third option. */
    description?: React.ReactNode;
    /** Sits inline after the title, baseline-aligned: a help popover trigger, a badge. */
    titleAdornment?: React.ReactNode;
    /** Trailing controls, bottom-aligned with the title block and wrapping under it when the bar is too narrow. */
    actions?: React.ReactNode;
    className?: string;
}

/**
 * The page header, in the only two shapes a page may take: a title on its own,
 * or a title with a description under it. Both carry the same breadcrumb, the
 * same type scale and the same trailing-action slot.
 *
 * Before this existed every page wrote its own: the tools family ran a
 * `24px -> 30px` title, the collection and player lists ran a flat `30px`, and
 * Stages ran a `27px -> 34px` title behind a kicker rule. The three read as
 * three different sites. A page now chooses BETWEEN THE TWO shapes here and
 * nothing else; anything that needs more goes in `actions`.
 */
export function PageHeader({ breadcrumb, breadcrumbLabel, title, description, titleAdornment, actions, className }: IPageHeaderProps): React.ReactElement {
    return (
        <div className={cn("flex flex-col", className)}>
            {breadcrumb && breadcrumb.length > 0 ? (
                <nav aria-label={breadcrumbLabel} className="mb-2.5 flex items-center gap-1.5 font-medium font-sans text-[12px] text-muted-foreground leading-none">
                    {breadcrumb.map((crumb, i) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: crumbs are a fixed positional trail, not a reorderable list
                        <Fragment key={i}>
                            {i > 0 ? <ChevronRight className="size-2.5" aria-hidden="true" /> : null}
                            <span className={cn(i === breadcrumb.length - 1 && "text-foreground")}>{crumb}</span>
                        </Fragment>
                    ))}
                </nav>
            ) : null}
            <div className="flex flex-wrap items-end justify-between gap-3">
                <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                        <h1 className="m-0 font-bold font-sans text-[24px] text-foreground leading-[1.1] tracking-tight sm:text-[30px]">{title}</h1>
                        {titleAdornment}
                    </div>
                    {description ? <p className="mt-1.5 max-w-2xl font-sans text-[13px] text-muted-foreground leading-normal sm:text-[13.5px]">{description}</p> : null}
                </div>
                {actions}
            </div>
        </div>
    );
}
