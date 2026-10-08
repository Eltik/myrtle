import { CurrencyIcon } from "frontend";

// The game's own icons for the currencies the pull planner counts. Decorative:
// the word beside each carries the meaning.
const NAMES = ["orundum", "originite", "permit", "tenPermit", "goldCert", "greenCert", "monthlyCard"] as const;

export const All = () => (
    <div className="flex items-center gap-3 p-4">
        {NAMES.map((n) => (
            <CurrencyIcon key={n} name={n} />
        ))}
    </div>
);

export const Large = () => (
    <div className="flex items-center gap-4 p-4">
        {NAMES.map((n) => (
            <CurrencyIcon key={n} name={n} className="size-10" />
        ))}
    </div>
);
