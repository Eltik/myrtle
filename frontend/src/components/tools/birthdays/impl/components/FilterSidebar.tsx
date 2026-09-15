import type * as React from "react";
import { Card, CardHeader, CardPanel, CardTitle } from "#/components/ui/card";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { IBirthdayFilters } from "../types";
import { FilterControls } from "./FilterControls";
import type { messages } from "./FilterSidebar.messages";

interface IFilterSidebarProps {
    filters: IBirthdayFilters;
    onChange: (next: IBirthdayFilters) => void;
    nations: [string, string][];
    matched: number;
    total: number;
    onReset: () => void;
}

/** Persistent filter rail shown at `lg` and up. */
export function FilterSidebar({ filters, onChange, nations, matched, total, onReset }: IFilterSidebarProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("tools");
    return (
        <aside className="hidden min-w-0 flex-col gap-4 lg:flex">
            <Card>
                <CardHeader className="pb-0">
                    <CardTitle className="text-[15px]">
                        {t("birthdays.sidebar.title")}
                        <span className="ml-1.5 font-medium font-mono text-[11px] text-muted-foreground">{t("birthdays.sidebar.count", { matched, total })}</span>
                    </CardTitle>
                </CardHeader>
                <CardPanel className="pt-0">
                    <FilterControls filters={filters} onChange={onChange} nations={nations} />
                </CardPanel>
                <div className="flex items-baseline justify-between border-border border-t px-6 py-3.5">
                    <div>
                        <div className="font-bold font-sans text-[22px] text-foreground tabular-nums leading-none tracking-tight">{matched}</div>
                        <div className="mt-1 font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em]">{t("birthdays.sidebar.matched")}</div>
                    </div>
                    <button type="button" className="cursor-pointer font-medium text-[12px] text-primary transition-colors hover:text-primary/80" onClick={onReset}>
                        {t("birthdays.sidebar.reset")}
                    </button>
                </div>
            </Card>
        </aside>
    );
}
