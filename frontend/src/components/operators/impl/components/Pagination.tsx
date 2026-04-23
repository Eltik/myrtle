interface IPaginationProps {
    currentPage: number;
    totalPages: number;
    onPageChange: (page: number) => void;
}

export function Pagination({ currentPage, totalPages, onPageChange }: IPaginationProps) {
    if (totalPages <= 1) return null;
    return <></>;
}
