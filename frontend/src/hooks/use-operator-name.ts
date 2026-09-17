import { useSelector } from "@tanstack/react-store";
import { useCallback } from "react";
import { type INamedOperator, operatorDisplayName } from "#/lib/operators/display-name";
import { themeStore } from "#/lib/theme/store";

/**
 * `(op) => string` bound to the Latin-names appearance preference. Subscribes
 * to that one field only, so a theme or accent change does not re-render the
 * operator lists that call it.
 */
export function useOperatorName(): (op: INamedOperator) => string {
    const latinNames = useSelector(themeStore, (s) => s.latinNames);
    return useCallback((op: INamedOperator) => operatorDisplayName(op, latinNames), [latinNames]);
}
