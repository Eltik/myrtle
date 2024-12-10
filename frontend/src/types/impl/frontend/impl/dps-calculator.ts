import type { Operator } from "../../api/static/operator";

export type OperatorSelectorProps = {
    operators: Operator[];
    selectedOperators: Operator[];

    isOpen: boolean;
    onClose: () => void;
    onSelect: (operators: Operator[]) => void;
};
