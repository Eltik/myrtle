import type * as React from "react";
import { itemIcon } from "#/components/operators/detail/impl/assets";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { IRowBalance } from "../plan";
import type { messages } from "./Primes.messages";

const OP_ICON = itemIcon("4002", "DIAMOND", null);

export function OpIcon({ className }: { className?: string }): React.ReactElement {
    return <img src={OP_ICON} alt="Originite Prime" title="Originite Prime" className={cn("inline-block size-4 object-contain align-[-3px]", className)} />;
}

export function Op({ value, sign, className }: { value: number; sign?: "+" | "-"; className?: string }): React.ReactElement {
    return (
        <span className={cn("inline-flex items-center gap-1 font-mono tabular-nums", className)}>
            {sign}
            {value}
            <OpIcon className="size-3.5" />
        </span>
    );
}

export function Calcs({ total, inline = false, className }: { total: IRowBalance | undefined; inline?: boolean; className?: string }): React.ReactElement {
    const t: TypedT<typeof messages> = useT("tools");
    const balance = total?.balance ?? 0;
    return (
        <dl className={cn("m-0 grid grid-cols-[auto_auto] gap-x-3 gap-y-0.5 text-[11px]", inline && "max-sm:grid-cols-[auto_auto_auto_auto_auto_auto] max-sm:gap-x-2", className)}>
            <dt className="font-sans text-muted-foreground">{t("release.primes.income")}</dt>
            <dd className="m-0 text-right text-emerald-500">
                <Op value={total?.income ?? 0} sign="+" />
            </dd>
            <dt className="font-sans text-muted-foreground">{t("release.primes.expenses")}</dt>
            <dd className="m-0 text-right text-rose-400">
                <Op value={total?.expense ?? 0} sign="-" />
            </dd>
            <dt className="font-sans font-semibold text-foreground">{t("release.primes.balance")}</dt>
            <dd className={cn("m-0 text-right font-semibold", balance < 0 ? "text-destructive-foreground" : "text-foreground")}>
                <Op value={balance} />
            </dd>
        </dl>
    );
}
