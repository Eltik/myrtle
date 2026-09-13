import { PaginationCompact } from "frontend";

const noop = () => {};

// The prev/next pair that sits in the results-summary row above the list, so a
// page is reachable without scrolling past the cards to the full pager
// (`ListPagination`). 438 operators at 12 per page is 37 pages.

export const MiddleOfList = () => <PaginationCompact currentPage={19} totalPages={37} onPageChange={noop} />;

/** Previous is disabled on the first page. */
export const FirstPage = () => <PaginationCompact currentPage={1} totalPages={37} onPageChange={noop} />;

/** Next is disabled on the last page. */
export const LastPage = () => <PaginationCompact currentPage={37} totalPages={37} onPageChange={noop} />;

/** In place: right of the "Showing … of … operators" summary in `Operators.tsx`.
 *  (With one page or fewer the component renders nothing.) */
export const InResultsRow = () => (
    <div className="flex w-full flex-wrap items-center justify-between gap-3 font-medium font-sans text-[12.5px] text-muted-foreground leading-none">
        <span>
            Showing <strong className="text-foreground">217</strong> to <strong className="text-foreground">228</strong> of <strong className="text-foreground">438</strong> operators
        </span>
        <div className="ml-auto flex items-center gap-3">
            <span className="hidden font-mono text-[11px] text-muted-foreground uppercase leading-none tracking-[0.08em] md:inline">Hover for preview · Click to open</span>
            <PaginationCompact currentPage={19} totalPages={37} onPageChange={noop} />
        </div>
    </div>
);
