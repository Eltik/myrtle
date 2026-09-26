import * as React from "react";

const AutoTranslateContext = React.createContext<boolean>(true);

export const AutoTranslateProvider = AutoTranslateContext.Provider;

export function useAutoTranslate(): boolean {
    return React.useContext(AutoTranslateContext);
}
