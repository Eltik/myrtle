import { ChevronLeft, ChevronRight } from "lucide-react";
import { cn } from "#/lib/utils";

interface IPaginationProps {
    currentPage: number;
    totalPages: number;
    onPageChange: (page: number) => void;
}

function getPageNumbers(current: number, total: number): (number | "ellipsis")[] {
    if (total <= 7) return Array.from({ length: total }, (_, i) => i + 1);
    const pages: (number | "ellipsis")[] = [1];
    const siblingStart = Math.max(2, current - 1);
    const siblingEnd = Math.min(total - 1, current + 1);
    if (siblingStart > 2) pages.push("ellipsis");
    for (let i = siblingStart; i <= siblingEnd; i++) pages.push(i);
    if (siblingEnd < total - 1) pages.push("ellipsis");
    pages.push(total);
    return pages;
}

export function Pagination({ currentPage, totalPages, onPageChange }: IPaginationProps) {
    if (totalPages <= 1) return null;
    return <></>;
}
