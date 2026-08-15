import { ElitePromotionCard } from "frontend";

export const Veteran = () => (
    <div className="w-full max-w-md">
        <ElitePromotionCard eliteBreakdown={{ e0: 58, e1: 62, e2: 148, total: 268 }} />
    </div>
);

export const EarlyAccount = () => (
    <div className="w-full max-w-md">
        <ElitePromotionCard eliteBreakdown={{ e0: 33, e1: 11, e2: 3, total: 47 }} />
    </div>
);

export const FullyPromoted = () => (
    <div className="w-full max-w-md">
        <ElitePromotionCard eliteBreakdown={{ e0: 0, e1: 0, e2: 324, total: 324 }} />
    </div>
);

export const InStatsGrid = () => (
    <div className="grid w-full max-w-3xl gap-3 sm:grid-cols-2">
        <ElitePromotionCard eliteBreakdown={{ e0: 58, e1: 62, e2: 148, total: 268 }} />
        <ElitePromotionCard eliteBreakdown={{ e0: 79, e1: 41, e2: 16, total: 136 }} />
    </div>
);
