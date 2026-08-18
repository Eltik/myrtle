import { createContext, useContext } from "react";
import type { IOptimizerApi } from "#/lib/base/use-optimizer";

const OptimizerContext = createContext<IOptimizerApi | null>(null);

export const OptimizerProvider = OptimizerContext.Provider;

export function useOptimizerApi(): IOptimizerApi {
    const api = useContext(OptimizerContext);
    if (!api) throw new Error("useOptimizerApi must be used inside an OptimizerProvider");
    return api;
}
