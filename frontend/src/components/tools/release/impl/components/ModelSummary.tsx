import type * as React from "react";
import { type TypedRichT, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { Backtest } from "#/types/generated/Backtest";
import type { LagModel } from "#/types/generated/LagModel";
import type { YearlyModel } from "#/types/generated/YearlyModel";
import { formatDays, formatPercent } from "../helpers";
import type { messages } from "./ModelSummary.messages";

type ModelT = TypedT<typeof messages>;
type ModelRichT = TypedRichT<typeof messages>;

interface IModelSummaryProps {
    model: LagModel | null | undefined;
    backtest?: Backtest | null;
    yearly?: YearlyModel | null;
}

function yearlyLabel(types: string[], t: ModelT): string {
    const names = types.map((type) => (type === "APRIL_FOOL" ? t("release.model.aprilFools") : type));
    return names.join(", ");
}

export function ModelSummary({ model, backtest, yearly }: IModelSummaryProps): React.ReactElement | null {
    const t: ModelT = useT("tools");
    const rt: ModelRichT = useRichT("tools");
    if (!model) return null;
    if (model.n === 0) return <p className="m-0 font-sans text-[12.5px] text-muted-foreground leading-normal">{t("release.model.empty")}</p>;
    return (
        <p className="m-0 font-sans text-[12.5px] text-muted-foreground leading-normal">
            {rt("release.model.median", {
                days: <span className="font-medium text-foreground tabular-nums">{t("release.model.days", { days: formatDays(model.medianDays) })}</span>,
                count: model.n,
                p25: formatDays(model.p25Days),
                p75: formatDays(model.p75Days),
            })}
            {backtest &&
                backtest.n > 0 &&
                rt("release.model.backtest", {
                    days: <span className="font-medium text-foreground tabular-nums">{t("release.model.days", { days: formatDays(backtest.medianAbsErrDays) })}</span>,
                    count: backtest.n,
                    rate: <span className="font-medium text-foreground tabular-nums">{formatPercent(backtest.bandHitRate)}</span>,
                })}
            .
            {yearly && yearly.model.n > 0 && (
                <>
                    {" "}
                    {rt("release.model.yearly", {
                        types: yearlyLabel(yearly.types, t),
                        days: <span className="font-medium text-foreground tabular-nums">{t("release.model.days", { days: formatDays(yearly.model.medianDays) })}</span>,
                        count: yearly.model.n,
                    })}
                </>
            )}
        </p>
    );
}
