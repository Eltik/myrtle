import { Button, Pagination, PaginationContent, PaginationEllipsis, PaginationItem, PaginationLink, PaginationNext, PaginationPrevious } from "frontend";

export const ActiveAndInactive = () => (
    <Pagination>
        <PaginationContent>
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
        </PaginationContent>
    </Pagination>
);

export const RenderedAsButtons = () => (
    <Pagination>
        <PaginationContent className="w-full flex-nowrap gap-1 sm:justify-center">
            <PaginationItem>
                <PaginationPrevious render={<Button className="h-9 gap-1.5 px-3 sm:h-8" size="sm" variant="outline" />} />
            </PaginationItem>
            <PaginationItem>
                <PaginationLink render={<Button className="size-9 sm:size-8" size="icon" variant="outline" />}>1</PaginationLink>
            </PaginationItem>
            <PaginationItem>
                <PaginationLink render={<Button className="size-9 sm:size-8" size="icon" variant="outline" />}>2</PaginationLink>
            </PaginationItem>
            <PaginationItem>
                <PaginationLink isActive render={<Button className="pointer-events-none size-9 sm:size-8" size="icon" variant="default" />}>
                    3
                </PaginationLink>
            </PaginationItem>
            <PaginationItem>
                <PaginationLink render={<Button className="size-9 sm:size-8" size="icon" variant="outline" />}>4</PaginationLink>
            </PaginationItem>
            <PaginationItem>
                <PaginationNext render={<Button className="h-9 gap-1.5 px-3 sm:h-8" size="sm" variant="outline" />} />
            </PaginationItem>
        </PaginationContent>
    </Pagination>
);

export const TextSizedLinks = () => (
    <Pagination>
        <PaginationContent>
            <PaginationItem>
                <PaginationLink size="default">Page 1</PaginationLink>
            </PaginationItem>
            <PaginationItem>
                <PaginationLink isActive size="default">
                    Page 2
                </PaginationLink>
            </PaginationItem>
            <PaginationItem>
                <PaginationLink size="default">Page 3</PaginationLink>
            </PaginationItem>
        </PaginationContent>
    </Pagination>
);
