import { ListPagination } from "frontend";

const noop = () => {};

/** The results summary the operator list renders just above its pager. */
const Results = ({ from, to, total }: { from: number; to: number; total: number }) => (
    <div className="flex flex-wrap items-center justify-between gap-3 font-medium font-sans text-[12.5px] text-muted-foreground leading-none">
        <span>
            Showing <strong className="text-foreground">{from}</strong> to <strong className="text-foreground">{to}</strong> of <strong className="text-foreground">{total}</strong> operators
        </span>
        <span className="hidden font-mono text-[11px] text-muted-foreground uppercase leading-none tracking-[0.08em] md:inline">Hover for preview · Click to open</span>
    </div>
);

export const MiddleOfList = () => (
    <div className="w-full">
        <Results from={217} to={228} total={438} />
        <ListPagination currentPage={19} totalPages={37} onPageChange={noop} />
    </div>
);

export const FirstPage = () => (
    <div className="w-full">
        <Results from={1} to={12} total={438} />
        <ListPagination currentPage={1} totalPages={37} onPageChange={noop} />
    </div>
);

export const LastPage = () => (
    <div className="w-full">
        <Results from={433} to={438} total={438} />
        <ListPagination currentPage={37} totalPages={37} onPageChange={noop} />
    </div>
);

export const ShortList = () => (
    <div className="w-full">
        <Results from={13} to={24} total={31} />
        <ListPagination currentPage={2} totalPages={3} onPageChange={noop} />
    </div>
);
