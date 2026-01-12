import type { SearchGrade, SearchSortBy } from "~/types/api";

// Sort options focused on user information (not leaderboard scores)
export const SORT_OPTIONS: { value: SearchSortBy; label: string }[] = [
    { value: "nickname", label: "Name" },
    { value: "level", label: "Level" },
    { value: "updated_at", label: "Recently Updated" },
    { value: "created_at", label: "Date Joined" },
    { value: "register_ts", label: "Account Age" },
];

export const GRADES: { value: SearchGrade | "all"; label: string }[] = [
    { value: "all", label: "All Grades" },
    { value: "S", label: "S Grade" },
    { value: "A", label: "A Grade" },
    { value: "B", label: "B Grade" },
    { value: "C", label: "C Grade" },
    { value: "D", label: "D Grade" },
    { value: "F", label: "F Grade" },
];
