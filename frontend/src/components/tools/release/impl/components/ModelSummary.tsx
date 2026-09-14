import type * as React from "react";
import type { Backtest } from "#/types/generated/Backtest";
import type { LagModel } from "#/types/generated/LagModel";
import type { YearlyModel } from "#/types/generated/YearlyModel";
import { formatDays, formatPercent } from "../helpers";

interface IModelSummaryProps {
    model: LagModel | null | undefined;
    backtest?: Backtest | null;
    yearly?: YearlyModel | null;
}

function yearlyLabel(types: string[]): string {
    const names = types.map((t) => (t === "APRIL_FOOL" ? "April Fools" : t));
    return names.join(", ");
}

export function ModelSummary({ model, backtest, yearly }: IModelSummaryProps): React.ReactElement | null {
    if (!model) return null;
    if (model.n === 0) return <p className="m-0 font-sans text-[12.5px] text-muted-foreground leading-normal">Lag model: no EN releases in the window yet, so nothing is estimated.</p>;
    return (
        <p className="m-0 font-sans text-[12.5px] text-muted-foreground leading-normal">
            Lag model: median <span className="font-medium text-foreground tabular-nums">{formatDays(model.medianDays)} d</span> over the last {model.n} EN releases (p25 {formatDays(model.p25Days)}, p75 {formatDays(model.p75Days)})
            {backtest && backtest.n > 0 && (
                <>
                    , backtest median abs error <span className="font-medium text-foreground tabular-nums">{formatDays(backtest.medianAbsErrDays)} d</span> over {backtest.n}, band hit rate <span className="font-medium text-foreground tabular-nums">{formatPercent(backtest.bandHitRate)}</span>
                </>
            )}
            .
            {yearly && yearly.model.n > 0 && (
                <>
                    {" "}
                    {yearlyLabel(yearly.types)}: <span className="font-medium text-foreground tabular-nums">{formatDays(yearly.model.medianDays)} d</span> over {yearly.model.n}, a year rather than a season.
                </>
            )}
        </p>
    );
}
