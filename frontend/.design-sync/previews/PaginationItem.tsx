import { Pagination, PaginationContent, PaginationEllipsis, PaginationItem, PaginationLink, PaginationNext, PaginationPrevious } from "frontend";

export const PageItems = () => (
    <Pagination>
        <PaginationContent>
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
                <PaginationEllipsis />
            </PaginationItem>
            <PaginationItem>
                <PaginationLink>12</PaginationLink>
            </PaginationItem>
            <PaginationItem>
                <PaginationNext />
            </PaginationItem>
        </PaginationContent>
    </Pagination>
);

export const WithSummaryItem = () => (
    <Pagination className="max-w-sm">
        <PaginationContent className="w-full flex-nowrap gap-1 sm:justify-center">
            <PaginationItem>
                <PaginationPrevious />
            </PaginationItem>
            <PaginationItem className="flex flex-1 justify-center">
                <span className="inline-flex h-9 items-center justify-center px-3 font-medium font-sans text-muted-foreground text-sm">
                    Page <strong className="mx-1 text-foreground">6</strong> of <strong className="ml-1 text-foreground">27</strong>
                </span>
            </PaginationItem>
            <PaginationItem>
                <PaginationNext />
            </PaginationItem>
        </PaginationContent>
    </Pagination>
);
