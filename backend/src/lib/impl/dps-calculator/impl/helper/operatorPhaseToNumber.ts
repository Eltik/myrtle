import { OperatorPhase } from "../../../../../types/impl/lib/impl/local/impl/gamedata/impl/operators";

export const operatorPhaseToNumber = (phase: OperatorPhase): number => {
    switch (phase) {
        case OperatorPhase.ELITE_0:
            return 0;
        case OperatorPhase.ELITE_1:
            return 1;
        case OperatorPhase.ELITE_2:
            return 2;
        default:
            throw new Error(`Unknown operator phase ${phase}`);
    }
};
