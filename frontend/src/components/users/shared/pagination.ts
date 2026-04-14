export type PaginationItem = { type: "page"; value: number } | { type: "ellipsis"; position: "start" | "end" };

export function generatePaginationItems(currentPage: number, totalPages: number): PaginationItem[] {
    const items: PaginationItem[] = [];

    if (totalPages <= 7) {
        for (let i = 1; i <= totalPages; i++) items.push({ type: "page", value: i });
        return items;
    }

    items.push({ type: "page", value: 1 });

    if (currentPage > 3) {
        items.push({ type: "ellipsis", position: "start" });
    }

    const start = Math.max(2, currentPage - 1);
    const end = Math.min(totalPages - 1, currentPage + 1);

    for (let i = start; i <= end; i++) {
        items.push({ type: "page", value: i });
    }

    if (currentPage < totalPages - 2) {
        items.push({ type: "ellipsis", position: "end" });
    }

    items.push({ type: "page", value: totalPages });

    return items;
}
