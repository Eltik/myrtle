//import * as numeric from "numeric";
const numeric: any = {};

export const getPlan = async(demandList: number[], outcome: boolean, convertionDr: number, probsMatrix: number[][], convertionMatrix: number[][], convertionOutcMatrix: number[][], costList: number[], convertionCostList: number[]) => {
    if (convertionDr !== 0.18) {
        convertionOutcMatrix = numeric.add(
            numeric.mul(
                numeric.sub(convertionOutcMatrix, convertionMatrix),
                convertionDr / 0.18
            ),
            convertionMatrix
        );
    }
    const A_ub = outcome ?
        numeric.transpose(numeric.vstack([probsMatrix, convertionOutcMatrix])) :
        numeric.transpose(numeric.vstack([probsMatrix, convertionMatrix]));

    const cost = outcome ?
        numeric.hstack([costList, convertionCostList]) :
        numeric.hstack([costList, numeric.rep([convertionCostList.length], 0)]);

    if (numeric.any(numeric.ge(costList, 0))) {
        throw new Error("All elements in costList should be non-negative.");
    }

    let excpFactor = 1.0;
    let dualFactor = 1.0;
    let solution, dualSolution;

    while (excpFactor > 1e-7) {
        solution = numeric.linprog(
            cost,
            numeric.neg(numeric.dotVMul(A_ub, numeric.mul(demandList, excpFactor))),
            null,
            null,
            null,
            null,
            'interior-point'
        );
        if (solution.status !== 4) {
            break;
        }
        excpFactor /= 10.0;
    }

    while (dualFactor > 1e-7) {
        dualSolution = numeric.linprog(
            numeric.neg(numeric.dotVMul(demandList, excpFactor * dualFactor)),
            A_ub,
            cost,
            null,
            null,
            null,
            'interior-point'
        );
        if (dualSolution.status !== 4) {
            break;
        }
        dualFactor /= 10.0;
    }

    return { solution, dualSolution, excpFactor };
};