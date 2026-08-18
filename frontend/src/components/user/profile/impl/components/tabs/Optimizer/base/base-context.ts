import { createContext, useContext } from "react";
import type { IOptimizerAPI } from "#/lib/base/use-optimizer";

const BaseOptimizerContext = createContext<IOptimizerAPI | null>(null);

export const BaseOptimizerProvider = BaseOptimizerContext.Provider;

export function useBaseOptimizer(): IOptimizerAPI {
    const api = useContext(BaseOptimizerContext);
    if (!api) throw new Error("useBaseOptimizer must be used inside a BaseOptimizerProvider");
    return api;
}
