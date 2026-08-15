import { Pagination, PaginationContent, PaginationEllipsis, PaginationItem, PaginationLink, PaginationNext, PaginationPrevious } from "frontend";

export const CenteredRow = () => (
    <Pagination>
        <PaginationContent>
            <PaginationItem>
                <PaginationPrevious />
            </PaginationItem>
            <PaginationItem>
                <PaginationLink>1</PaginationLink>
            </PaginationItem>
            <PaginationItem>
                <PaginationLink isActive>2</PaginationLink>
            </PaginationItem>
            <PaginationItem>
                <PaginationLink>3</PaginationLink>
            </PaginationItem>
            <PaginationItem>
                <PaginationEllipsis />
            </PaginationItem>
            <PaginationItem>
                <PaginationLink>18</PaginationLink>
            </PaginationItem>
            <PaginationItem>
                <PaginationNext />
            </PaginationItem>
        </PaginationContent>
    </Pagination>
);

export const UnderneathAResultCount = () => (
    <div className="flex w-full max-w-2xl flex-col gap-3">
        <div className="flex items-center justify-between gap-4 font-sans text-muted-foreground text-sm">
            <span>
                Showing <span className="font-medium text-foreground tabular-nums">41–60</span> of <span className="font-medium text-foreground tabular-nums">532</span> operators
            </span>
            <span className="font-mono text-xs">sorted by rarity</span>
        </div>
        <Pagination className="mt-0">
            <PaginationContent className="w-full flex-nowrap gap-1 sm:justify-center">
                <PaginationItem>
                    <PaginationPrevious />
                </PaginationItem>
                <PaginationItem>
                    <PaginationLink>1</PaginationLink>
                </PaginationItem>
                <PaginationItem>
                    <PaginationLink>2</PaginationLink>
                </PaginationItem>
                <PaginationItem>
                    <PaginationLink isActive>3</PaginationLink>
                </PaginationItem>
                <PaginationItem>
                    <PaginationLink>4</PaginationLink>
                </PaginationItem>
                <PaginationItem>
                    <PaginationEllipsis />
                </PaginationItem>
                <PaginationItem>
                    <PaginationLink>27</PaginationLink>
                </PaginationItem>
                <PaginationItem>
                    <PaginationNext />
                </PaginationItem>
            </PaginationContent>
        </Pagination>
    </div>
);
