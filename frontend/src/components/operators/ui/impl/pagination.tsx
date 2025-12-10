"use client";

import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight } from "lucide-react";
import { useState } from "react";
import { cn } from "~/lib/utils";

interface PaginationProps {
    currentPage: number;
    totalPages: number;
    onPageChange: (page: number) => void;
}

type PageItem = number | "ellipsis-start" | "ellipsis-end";

function getPages(currentPage: number, totalPages: number, maxVisible: number): PageItem[] {
    const pages: PageItem[] = [];

    if (totalPages <= maxVisible) {
        for (let i = 1; i <= totalPages; i++) pages.push(i);
        return pages;
    }

    pages.push(1);
    const sideCount = Math.floor((maxVisible - 3) / 2);

    if (currentPage <= sideCount + 2) {
        for (let i = 2; i <= Math.min(maxVisible - 2, totalPages - 1); i++) pages.push(i);
        if (totalPages > maxVisible - 1) pages.push("ellipsis-end");
    } else if (currentPage >= totalPages - sideCount - 1) {
        pages.push("ellipsis-start");
        for (let i = Math.max(totalPages - maxVisible + 3, 2); i <= totalPages - 1; i++) pages.push(i);
    } else {
        pages.push("ellipsis-start");
        for (let i = currentPage - sideCount; i <= currentPage + sideCount; i++) pages.push(i);
        pages.push("ellipsis-end");
    }

    pages.push(totalPages);
    return pages;
}

function PageButton({ page, currentPage, onClick }: { page: number; currentPage: number; onClick: () => void }) {
    return (
        <button
            className={cn("flex h-9 w-9 items-center justify-center rounded-lg border text-sm transition-colors", currentPage === page ? "border-primary bg-primary text-primary-foreground" : "border-border bg-secondary/50 text-muted-foreground hover:bg-secondary hover:text-foreground")}
            onClick={onClick}
            type="button"
        >
            {page}
        </button>
    );
}

function NavButton({ onClick, disabled, title, children }: { onClick: () => void; disabled: boolean; title: string; children: React.ReactNode }) {
    return (
        <button className="flex h-9 w-9 items-center justify-center rounded-lg border border-border bg-secondary/50 text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50" disabled={disabled} onClick={onClick} title={title} type="button">
            {children}
        </button>
    );
}

export function Pagination({ currentPage, totalPages, onPageChange }: PaginationProps) {
    const [pageInput, setPageInput] = useState(currentPage.toString());

    const handlePageChange = (page: number) => {
        const newPage = Math.max(1, Math.min(page, totalPages));
        onPageChange(newPage);
        setPageInput(newPage.toString());
    };

    const handleInputSubmit = () => {
        const page = Number.parseInt(pageInput, 10);
        if (!Number.isNaN(page) && page >= 1 && page <= totalPages) {
            handlePageChange(page);
        } else {
            setPageInput(currentPage.toString());
        }
    };

    if (totalPages <= 1) return null;

    return (
        <div className="flex flex-col items-center gap-4 pt-6 sm:flex-row sm:justify-between">
            {/* Page info and go-to input */}
            <div className="flex flex-wrap items-center justify-center gap-3">
                <span className="text-muted-foreground text-sm">
                    Page {currentPage} of {totalPages}
                </span>
                <div className="flex items-center gap-1.5">
                    <span className="text-muted-foreground text-sm">Go to:</span>
                    <input
                        className="h-8 w-14 rounded-md border border-border bg-secondary/50 px-2 text-center text-foreground text-sm focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
                        max={totalPages}
                        min={1}
                        onBlur={handleInputSubmit}
                        onChange={(e) => setPageInput(e.target.value)}
                        onKeyDown={(e) => {
                            if (e.key === "Enter") handleInputSubmit();
                        }}
                        type="number"
                        value={pageInput}
                    />
                </div>
            </div>

            {/* Page buttons */}
            <div className="flex items-center gap-1">
                <NavButton disabled={currentPage === 1} onClick={() => handlePageChange(1)} title="First page">
                    <ChevronsLeft className="h-4 w-4" />
                </NavButton>
                <NavButton disabled={currentPage === 1} onClick={() => handlePageChange(currentPage - 1)} title="Previous page">
                    <ChevronLeft className="h-4 w-4" />
                </NavButton>

                {/* Mobile: fewer pages */}
                <div className="flex items-center gap-1 sm:hidden">
                    {getPages(currentPage, totalPages, 5).map((page) =>
                        page === "ellipsis-start" || page === "ellipsis-end" ? (
                            <span className="px-1 text-muted-foreground" key={page}>
                                ...
                            </span>
                        ) : (
                            <PageButton currentPage={currentPage} key={page} onClick={() => handlePageChange(page)} page={page} />
                        ),
                    )}
                </div>

                {/* Desktop: more pages */}
                <div className="hidden items-center gap-1 sm:flex">
                    {getPages(currentPage, totalPages, 7).map((page) =>
                        page === "ellipsis-start" || page === "ellipsis-end" ? (
                            <span className="px-1 text-muted-foreground" key={page}>
                                ...
                            </span>
                        ) : (
                            <PageButton currentPage={currentPage} key={page} onClick={() => handlePageChange(page)} page={page} />
                        ),
                    )}
                </div>

                <NavButton disabled={currentPage === totalPages} onClick={() => handlePageChange(currentPage + 1)} title="Next page">
                    <ChevronRight className="h-4 w-4" />
                </NavButton>
                <NavButton disabled={currentPage === totalPages} onClick={() => handlePageChange(totalPages)} title="Last page">
                    <ChevronsRight className="h-4 w-4" />
                </NavButton>
            </div>
        </div>
    );
}
