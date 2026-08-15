import { Pagination, PaginationContent, PaginationEllipsis, PaginationItem, PaginationLink, PaginationNext, PaginationPrevious } from "frontend";

export const InAPageBar = () => (
    <Pagination>
        <PaginationContent>
            <PaginationItem>
                <PaginationPrevious />
            </PaginationItem>
            <PaginationItem>
                <PaginationLink>1</PaginationLink>
            </PaginationItem>
            <PaginationItem>
                <PaginationEllipsis />
            </PaginationItem>
            <PaginationItem>
                <PaginationLink>11</PaginationLink>
            </PaginationItem>
            <PaginationItem>
                <PaginationLink isActive>12</PaginationLink>
            </PaginationItem>
            <PaginationItem>
                <PaginationLink>13</PaginationLink>
            </PaginationItem>
            <PaginationItem>
                <PaginationNext />
            </PaginationItem>
        </PaginationContent>
    </Pagination>
);

export const DisabledOnLastPage = () => (
    <Pagination>
        <PaginationContent>
            <PaginationItem>
                <PaginationPrevious />
            </PaginationItem>
            <PaginationItem>
                <PaginationLink>1</PaginationLink>
            </PaginationItem>
            <PaginationItem>
                <PaginationEllipsis />
            </PaginationItem>
            <PaginationItem>
                <PaginationLink>25</PaginationLink>
            </PaginationItem>
            <PaginationItem>
                <PaginationLink>26</PaginationLink>
            </PaginationItem>
            <PaginationItem>
                <PaginationLink isActive>27</PaginationLink>
            </PaginationItem>
            <PaginationItem>
                <PaginationNext aria-disabled className="pointer-events-none opacity-50" />
            </PaginationItem>
        </PaginationContent>
    </Pagination>
);
