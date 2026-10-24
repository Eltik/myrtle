export interface IQueryState {
    isPending: boolean;
    error: Error | null;
}

/** Whether requests wait or fail on the supporter values and, for class-limited supporters, the operator index. Pure, tested. */
export function teamBuffsGate({ enabled, supporters, scoped, index }: { enabled: boolean; supporters: IQueryState; scoped: boolean; index: IQueryState }): { waiting: boolean; error: Error | null } {
    if (!enabled) return { waiting: false, error: null };
    if (supporters.error) return { waiting: false, error: supporters.error };
    if (supporters.isPending) return { waiting: true, error: null };
    if (!scoped) return { waiting: false, error: null };
    if (index.error) return { waiting: false, error: index.error };
    return { waiting: index.isPending, error: null };
}
