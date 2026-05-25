import { ChevronsLeft, ChevronsRight, CornerDownLeft } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import { PaginationContent, PaginationEllipsis, PaginationItem, PaginationLink, PaginationNext, PaginationPrevious, Pagination as PaginationRoot } from "#/components/ui/pagination";
import { Popover, PopoverContent, PopoverTrigger } from "#/components/ui/popover";
import { cn } from "#/lib/utils";

interface IPaginationProps {
    currentPage: number;
    totalPages: number;
    onPageChange: (page: number) => void;
    className?: string;
}

const SLOT_WIDTH_PX = 36;
const FIXED_SLOTS = 5;
const PREV_NEXT_RESERVE_PX = 200;

type EllipsisItem = { kind: "ellipsis-start" | "ellipsis-end"; from: number; to: number };
type PaginationItemModel = number | EllipsisItem;

function generatePaginationRange(currentPage: number, totalPages: number, siblingCount: number): PaginationItemModel[] {
    const siblings = Math.max(1, siblingCount);

    const leftSiblingIndex = Math.max(currentPage - siblings, 1);
    const rightSiblingIndex = Math.min(currentPage + siblings, totalPages);

    const showLeftEllipsis = leftSiblingIndex > 2;
    const showRightEllipsis = rightSiblingIndex < totalPages - 1;

    const items: PaginationItemModel[] = [];

    if (totalPages >= 1) {
        items.push(1);
    }

    if (showLeftEllipsis) {
        items.push({ kind: "ellipsis-start", from: 2, to: leftSiblingIndex - 1 });
    } else if (leftSiblingIndex > 1) {
        for (let i = 2; i < leftSiblingIndex; i++) {
            items.push(i);
        }
    }

    for (let i = leftSiblingIndex; i <= rightSiblingIndex; i++) {
        if (i !== 1 && i !== totalPages) {
            items.push(i);
        }
    }

    if (showRightEllipsis) {
        items.push({ kind: "ellipsis-end", from: rightSiblingIndex + 1, to: totalPages - 1 });
    } else if (rightSiblingIndex < totalPages) {
        for (let i = rightSiblingIndex + 1; i < totalPages; i++) {
            items.push(i);
        }
    }

    if (totalPages > 1) {
        items.push(totalPages);
    }

    return items;
}

export function Pagination({ currentPage, totalPages, onPageChange, className }: IPaginationProps) {
    const contentRef = useRef<HTMLUListElement>(null);
    const [siblingCount, setSiblingCount] = useState(1);

    useEffect(() => {
        const el = contentRef.current;
        if (!el || typeof ResizeObserver === "undefined") {
            return;
        }

        const apply = (width: number) => {
            const available = width - PREV_NEXT_RESERVE_PX;
            if (available <= 0) {
                setSiblingCount((prev) => (prev === 1 ? prev : 1));
                return;
            }
            const slots = Math.max(FIXED_SLOTS, Math.floor(available / SLOT_WIDTH_PX));
            const next = Math.max(1, Math.floor((slots - FIXED_SLOTS) / 2) + 1);
            setSiblingCount((prev) => (prev === next ? prev : next));
        };

        apply(el.clientWidth);
        const observer = new ResizeObserver((entries) => {
            for (const entry of entries) {
                apply(entry.contentRect.width);
            }
        });
        observer.observe(el);
        return () => observer.disconnect();
    }, []);

    const paginationRange = useMemo(() => generatePaginationRange(currentPage, totalPages, siblingCount), [currentPage, totalPages, siblingCount]);

    const handlePrevious = useCallback(() => {
        if (currentPage > 1) {
            onPageChange(currentPage - 1);
        }
    }, [currentPage, onPageChange]);

    const handleNext = useCallback(() => {
        if (currentPage < totalPages) {
            onPageChange(currentPage + 1);
        }
    }, [currentPage, totalPages, onPageChange]);

    const handleFirst = useCallback(() => {
        if (currentPage !== 1) {
            onPageChange(1);
        }
    }, [currentPage, onPageChange]);

    const handleLast = useCallback(() => {
        if (currentPage !== totalPages) {
            onPageChange(totalPages);
        }
    }, [currentPage, totalPages, onPageChange]);

    const handlePageClick = useCallback(
        (page: number) => {
            if (page !== currentPage) {
                onPageChange(page);
            }
        },
        [currentPage, onPageChange],
    );

    if (totalPages <= 1) {
        return null;
    }

    return (
        <PaginationRoot className={cn("mt-6", className)}>
            <PaginationContent ref={contentRef} className="w-full flex-nowrap gap-1 sm:justify-center">
                <PaginationItem className="flex sm:hidden">
                    <Button variant="outline" size="default" disabled={currentPage <= 1} onClick={handleFirst} aria-label="Go to first page" className="aspect-square p-0">
                        <ChevronsLeft />
                    </Button>
                </PaginationItem>
                <PaginationItem>
                    <PaginationPrevious render={<Button variant="outline" size="sm" disabled={currentPage <= 1} onClick={handlePrevious} aria-label="Go to previous page" className="h-9 gap-1.5 px-3 sm:h-8" />} />
                </PaginationItem>

                {paginationRange.map((item) => {
                    if (typeof item === "object") {
                        const suggested = Math.floor((item.from + item.to) / 2);
                        return (
                            <PaginationItem key={item.kind} className="hidden sm:flex">
                                <JumpToPagePopover totalPages={totalPages} hiddenFrom={item.from} hiddenTo={item.to} suggestedPage={suggested} onJump={handlePageClick} />
                            </PaginationItem>
                        );
                    }

                    const isActive = item === currentPage;

                    return (
                        <PaginationItem key={item} className="hidden sm:flex">
                            <PaginationLink
                                isActive={isActive}
                                render={<Button variant={isActive ? "default" : "outline"} size="icon" onClick={() => handlePageClick(item)} aria-label={`Go to page ${item}`} aria-current={isActive ? "page" : undefined} className={cn("size-9 sm:size-8", isActive && "pointer-events-none")} />}
                            >
                                {item}
                            </PaginationLink>
                        </PaginationItem>
                    );
                })}

                <PaginationItem className="flex flex-1 justify-center sm:hidden">
                    <span className="inline-flex h-9 items-center justify-center px-3 font-medium font-sans text-muted-foreground text-sm">
                        Page <strong className="mx-1 text-foreground">{currentPage}</strong> of <strong className="ml-1 text-foreground">{totalPages}</strong>
                    </span>
                </PaginationItem>
                <PaginationItem>
                    <PaginationNext render={<Button variant="outline" size="sm" disabled={currentPage >= totalPages} onClick={handleNext} aria-label="Go to next page" className="h-9 gap-1.5 px-3 sm:h-8" />} />
                </PaginationItem>
                <PaginationItem className="flex sm:hidden">
                    <Button variant="outline" size="default" disabled={currentPage >= totalPages} onClick={handleLast} aria-label="Go to last page" className="aspect-square p-0">
                        <ChevronsRight />
                    </Button>
                </PaginationItem>
            </PaginationContent>
        </PaginationRoot>
    );
}

interface IJumpToPagePopoverProps {
    totalPages: number;
    hiddenFrom: number;
    hiddenTo: number;
    suggestedPage: number;
    onJump: (page: number) => void;
}

function JumpToPagePopover({ totalPages, hiddenFrom, hiddenTo, suggestedPage, onJump }: IJumpToPagePopoverProps) {
    const [open, setOpen] = useState(false);
    const [value, setValue] = useState(String(suggestedPage));
    const inputRef = useRef<HTMLInputElement>(null);

    useEffect(() => {
        if (open) {
            setValue(String(suggestedPage));
            const id = window.setTimeout(() => {
                inputRef.current?.focus();
                inputRef.current?.select();
            }, 0);
            return () => window.clearTimeout(id);
        }
        return;
    }, [open, suggestedPage]);

    const parsed = Number.parseInt(value, 10);
    const isValid = Number.isFinite(parsed) && parsed >= 1 && parsed <= totalPages;

    const commit = () => {
        if (!isValid) return;
        onJump(parsed);
        setOpen(false);
    };

    return (
        <Popover open={open} onOpenChange={setOpen}>
            <PopoverTrigger
                render={
                    <button
                        type="button"
                        aria-label={`Jump to a page between ${hiddenFrom} and ${hiddenTo}`}
                        className="inline-flex size-8 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring data-popup-open:bg-accent data-popup-open:text-foreground"
                    >
                        <PaginationEllipsis className="min-w-0" />
                    </button>
                }
            />
            <PopoverContent align="center" sideOffset={6} className="w-60">
                <div className="flex flex-col gap-2.5">
                    <div className="flex flex-col gap-0.5">
                        <span className="font-sans font-semibold text-foreground text-sm leading-none">Jump to page</span>
                        <span className="font-sans text-muted-foreground text-xs tabular-nums leading-none">
                            Hidden range: {hiddenFrom}–{hiddenTo} · Total {totalPages}
                        </span>
                    </div>
                    <form
                        onSubmit={(event) => {
                            event.preventDefault();
                            commit();
                        }}
                        className="flex items-center gap-1.5"
                    >
                        <Input
                            ref={inputRef}
                            type="number"
                            inputMode="numeric"
                            min={1}
                            max={totalPages}
                            value={value}
                            onChange={(event) => setValue(event.currentTarget.value)}
                            aria-label="Page number"
                            aria-invalid={value.length > 0 && !isValid}
                            className="flex-1 tabular-nums `[[type=number]]:[appearance:textfield] [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none"
                        />
                        <Button type="submit" size="icon" variant="default" disabled={!isValid} aria-label="Go to page" className="size-8 shrink-0">
                            <CornerDownLeft className="size-4" />
                        </Button>
                    </form>
                    <span className="font-sans text-[11px] text-muted-foreground leading-none">Enter a number from 1 to {totalPages}.</span>
                </div>
            </PopoverContent>
        </Popover>
    );
}
