import * as React from "react";

const STORAGE_KEY = "release.autoTranslate";

const AutoTranslateContext = React.createContext<boolean>(true);

export const AutoTranslateProvider = AutoTranslateContext.Provider;

export function useAutoTranslate(): boolean {
    return React.useContext(AutoTranslateContext);
}

export function useAutoTranslateSetting(): [boolean, (value: boolean) => void] {
    const [enabled, setEnabled] = React.useState(true);

    React.useEffect(() => {
        try {
            const stored = window.localStorage.getItem(STORAGE_KEY);
            if (stored === "0" || stored === "false") setEnabled(false);
        } catch {}
    }, []);

    const update = React.useCallback((value: boolean) => {
        setEnabled(value);
        try {
            window.localStorage.setItem(STORAGE_KEY, value ? "1" : "0");
        } catch {}
    }, []);

    return [enabled, update];
}
