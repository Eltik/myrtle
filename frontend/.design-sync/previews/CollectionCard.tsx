import { CollectionCard } from "frontend";

// Roster sizes as the Stats tab computes them: obtainable operators only,
// TOKEN/TRAP excluded.
export const Veteran = () => (
    <div className="w-full max-w-md">
        <CollectionCard collectionPercentage={80.97} totalAvailable={331} totalOwned={268} />
    </div>
);

export const NewDoctor = () => (
    <div className="w-full max-w-md">
        <CollectionCard collectionPercentage={14.2} totalAvailable={331} totalOwned={47} />
    </div>
);

export const NearComplete = () => (
    <div className="w-full max-w-md">
        <CollectionCard collectionPercentage={97.89} totalAvailable={331} totalOwned={324} />
    </div>
);

export const InStatsGrid = () => (
    <div className="grid w-full max-w-3xl gap-3 sm:grid-cols-2">
        <CollectionCard collectionPercentage={80.97} totalAvailable={331} totalOwned={268} />
        <CollectionCard collectionPercentage={41.09} totalAvailable={331} totalOwned={136} />
    </div>
);
