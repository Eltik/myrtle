import type { DPSOperator, OperatorParams } from "../../api/impl/dps-calculator";
import type { Operator } from "../../api/static/operator";

// Add SelectedDPSOperator type for use in OperatorListItemProps
export interface SelectedDPSOperator extends DPSOperator {
    instanceId: string;
    displayName: string;
}

export type OperatorSelectorProps = {
    operators: Operator[];
    selectedOperators: Operator[];

    isOpen: boolean;
    onClose: () => void;
    onSelect: (operators: Operator[]) => void;
};

export type OperatorListItemProps = {
    operator: SelectedDPSOperator;
    onParamsChange: (params: OperatorParams) => void;
    onRemove?: (instanceId: string) => void;
    onDuplicate?: (instanceId: string) => void;
};
