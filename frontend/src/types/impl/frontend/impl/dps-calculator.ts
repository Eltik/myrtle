import type { DPSOperator, OperatorParams } from "../../api/impl/dps-calculator";
import type { Operator } from "../../api/static/operator";

export type OperatorSelectorProps = {
    operators: Operator[];
    selectedOperators: Operator[];

    isOpen: boolean;
    onClose: () => void;
    onSelect: (operators: Operator[]) => void;
};

export type OperatorListItemProps = {
    operator: DPSOperator;
    onParamsChange: (params: OperatorParams) => void;
};
