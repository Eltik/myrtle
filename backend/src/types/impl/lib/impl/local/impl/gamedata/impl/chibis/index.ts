export interface RepoItem {
    name: string;
    path: string;
    contentType: string;
    children?: RepoItem[];
}

export interface CachedData {
    timestamp: number;
    data: RepoItem[];
}
