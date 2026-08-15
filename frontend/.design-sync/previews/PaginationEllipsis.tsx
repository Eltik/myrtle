import { Pagination, PaginationContent, PaginationEllipsis, PaginationItem, PaginationLink, PaginationNext, PaginationPrevious } from "frontend";

export const TrailingGap = () => (
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
                <PaginationLink>27</PaginationLink>
            </PaginationItem>
            <PaginationItem>
                <PaginationNext />
            </PaginationItem>
        </PaginationContent>
    </Pagination>
);

export const GapsOnBothSides = () => (
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
                <PaginationLink>13</PaginationLink>
            </PaginationItem>
            <PaginationItem>
                <PaginationLink isActive>14</PaginationLink>
            </PaginationItem>
            <PaginationItem>
                <PaginationLink>15</PaginationLink>
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
);
