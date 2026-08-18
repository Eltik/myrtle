import type { ComponentType } from "react";
import type { IRosterEntry } from "#/lib/api/user";
import type { IOperatorListItem } from "#/types/operators";
import { BaseOptimizer } from "./base/BaseOptimizer";

export interface IOptimizerProps {
    uid: string;
    roster: IRosterEntry[];
    operatorsStatic: IOperatorListItem[];
}

export interface IOptimizerDef {
    id: string;
    label: string;
    blurb: string;
    Component?: ComponentType<IOptimizerProps>;
}

export const OPTIMIZERS: IOptimizerDef[] = [
    {
        id: "base",
        label: "Base Optimizer",
        blurb: "Solve the best peak staffing for the RIIC, then plans a shift rotation.",
        Component: BaseOptimizer,
    },
    {
        id: "account",
        label: "Account Optimizer",
        blurb: "Where to spend next across the account - promotions, masteries and modules, ranked by what each actually returns.",
    },
];
