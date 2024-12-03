export function getBlackboard(
    blackboardArray: {
        key: string;
        value: number;
        valueStr?: string | null;
    }[],
) {
    const blackboard: Record<string, number> = {};
    blackboardArray.forEach((kv) => (blackboard[kv.key] = kv.value));
    return blackboard;
}
