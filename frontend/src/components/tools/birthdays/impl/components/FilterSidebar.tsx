import type * as React from "react";
import { Card, CardHeader, CardPanel, CardTitle } from "#/components/ui/card";
import { Kicker } from "#/components/ui/kicker";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { IBirthdayFilters } from "../types";
import { FilterControls } from "./FilterControls";
import type { messages } from "./FilterSidebar.messages";

interface IFilterSidebarProps {
    filters: IBirthdayFilters;
    onChange: (next: IBirthdayFilters) => void;
    nations: [string, string][];
    /** Class code -> the server's own class name. */
    classNames: ReadonlyMap<string, string>;
    matched: number;
    total: number;
    onReset: () => void;
}

/** Persistent filter rail shown at `lg` and up. */
export function FilterSidebar({ filters, onChange, nations, classNames, matched, total, onReset }: IFilterSidebarProps): React.ReactElement {
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
                    <FilterControls filters={filters} onChange={onChange} nations={nations} classNames={classNames} />
                </CardPanel>
                <div className="flex items-baseline justify-between border-border border-t px-6 py-3.5">
                    <div>
                        <div className="font-bold font-sans text-[22px] text-foreground tabular-nums leading-none tracking-tight">{matched}</div>
                        <Kicker className="mt-1 block">{t("birthdays.sidebar.matched")}</Kicker>
                    </div>
                    <button
                        type="button"
                        className="pointer-coarse:after:-translate-1/2 relative cursor-pointer rounded-sm font-medium text-[12px] text-primary underline-offset-2 pointer-coarse:after:absolute pointer-coarse:after:top-1/2 pointer-coarse:after:left-1/2 pointer-coarse:after:size-11 hover:underline"
                        onClick={onReset}
                    >
                        {t("birthdays.sidebar.reset")}
                    </button>
                </div>
            </Card>
        </aside>
    );
}
