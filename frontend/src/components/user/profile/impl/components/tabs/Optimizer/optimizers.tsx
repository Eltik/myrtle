import type { ComponentType } from "react";
import type { IRosterEntry } from "#/lib/api/user";
import type { IOperatorListItem } from "#/types/operators";
import { BaseOptimizer } from "./base/BaseOptimizer";
import type { messages as optimizerMessages } from "./optimizers.messages";

export interface IOptimizerProps {
    uid: string;
    roster: IRosterEntry[];
    operatorsStatic: IOperatorListItem[];
}

/** A key in `optimizers.messages.ts`; resolved by the tab strip. */
export type OptimizerMessageKey = keyof typeof optimizerMessages & string;

export interface IOptimizerDef {
    id: string;
    labelKey: OptimizerMessageKey;
    blurbKey: OptimizerMessageKey;
    Component?: ComponentType<IOptimizerProps>;
}

export const OPTIMIZERS: IOptimizerDef[] = [
    {
        id: "base",
        labelKey: "profile.optimizer.base.label",
        blurbKey: "profile.optimizer.base.blurb",
        Component: BaseOptimizer,
    },
    {
        id: "account",
        labelKey: "profile.optimizer.account.label",
        blurbKey: "profile.optimizer.account.blurb",
    },
];
