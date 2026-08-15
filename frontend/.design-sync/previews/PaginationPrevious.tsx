import { Pagination, PaginationContent, PaginationEllipsis, PaginationItem, PaginationLink, PaginationNext, PaginationPrevious } from "frontend";

export const InAPageBar = () => (
    <Pagination>
        <PaginationContent>
            <PaginationItem>
                <PaginationPrevious />
            </PaginationItem>
            <PaginationItem>
                <PaginationLink>4</PaginationLink>
            </PaginationItem>
            <PaginationItem>
                <PaginationLink isActive>5</PaginationLink>
            </PaginationItem>
            <PaginationItem>
                <PaginationLink>6</PaginationLink>
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

export const DisabledOnFirstPage = () => (
    <Pagination>
        <PaginationContent>
            <PaginationItem>
                <PaginationPrevious aria-disabled className="pointer-events-none opacity-50" />
            </PaginationItem>
            <PaginationItem>
                <PaginationLink isActive>1</PaginationLink>
            </PaginationItem>
            <PaginationItem>
                <PaginationLink>2</PaginationLink>
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
