import { OperatorsList } from "frontend";
import { type ReactNode, useState } from "react";

// `OperatorsList` reads the whole operator index, the voice-actor table, per-user
// notes and population ownership through `createServerFn` fetchers. Those are
// stubbed in the preview bundle and the preview QueryClient never retries, so the
// page resolves straight to its zero-result branch — toolbar, sort/page controls
// and the "no operators match your filters" panel are the real, live render.

/** View mode and sidebar visibility live in localStorage, not in props — and the
 *  capture browser keeps localStorage between stories, so every story sets both. */
const Prefs = ({ filters, view, children }: { filters: "0" | "1"; view: string; children: ReactNode }) => {
    useState(() => {
        localStorage.setItem("operators:filters-visible", filters);
        localStorage.setItem("operators:view-mode", view);
        localStorage.setItem("operators:items-per-page", "30");
        return null;
    });
    return <>{children}</>;
};

export const NoResults = () => (
    <Prefs filters="0" view="grid">
        <OperatorsList />
    </Prefs>
);

export const WithFilterSidebar = () => (
    <Prefs filters="1" view="grid">
        <OperatorsList />
    </Prefs>
);
