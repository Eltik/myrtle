import { ModelSummary } from "frontend";
import { type ReactNode, useEffect, useRef } from "react";

// The "how are these dates estimated?" disclosure on the Events and Banners tabs.
// Figures are the live /release/lag payload: the last 10 CN-to-EN lags (median
// 158.0 days, p25 156.4, p75 162.2), a backtest over 62 past events (median
// error 4.9 days, 37% inside the band), and the yearly April Fools' model (4
// samples, 365.4 days).
const MODEL = { window: 10, n: 10, medianDays: 157.97916666666666, p25Days: 156.35416666666666, p75Days: 162.19791666666666, samples: [] };
const BACKTEST = { window: 10, n: 62, medianAbsErrDays: 4.9375, p75AbsErrDays: 8.6875, p90AbsErrDays: 17.658333333333324, maxAbsErrDays: 49.5, bandHitRate: 0.3709677419354839 };
const YEARLY = { types: ["APRIL_FOOL"], model: { window: 4, n: 4, medianDays: 365.375, p25Days: 365.3541666666667, p75Days: 365.625, samples: [] } };

function Opened({ children }: { children: ReactNode }) {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        const d = ref.current?.querySelector("details");
        if (d) d.open = true;
    }, []);
    return (
        <div ref={ref} className="w-full max-w-2xl p-4">
            {children}
        </div>
    );
}

// Closed, as it ships: the question is the control.
export const Closed = () => (
    <div className="w-full max-w-2xl p-4">
        <ModelSummary model={MODEL} backtest={BACKTEST} yearly={YEARLY} />
    </div>
);

// Opened: median lag, backtest error and hit rate, and the yearly model.
export const Expanded = () => (
    <Opened>
        <ModelSummary model={MODEL} backtest={BACKTEST} yearly={YEARLY} />
    </Opened>
);

// Lag model only (the Skins tab passes no backtest or yearly model).
export const LagOnly = () => (
    <Opened>
        <ModelSummary model={MODEL} />
    </Opened>
);

// No samples yet.
export const NoSamples = () => (
    <div className="w-full max-w-2xl p-4">
        <ModelSummary model={{ window: 10, n: 0, medianDays: 0, p25Days: 0, p75Days: 0, samples: [] }} />
    </div>
);
