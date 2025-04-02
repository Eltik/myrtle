export interface GitHubCommit {
    commit: {
        author: {
            date: string;
            name: string;
        };
        message: string;
    };
    sha: string;
}
