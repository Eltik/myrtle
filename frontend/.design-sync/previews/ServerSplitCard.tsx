import { ServerSplitCard } from "frontend";

const TOP_250 = [
    { server: "EN", players: 96 },
    { server: "JP", players: 54 },
    { server: "CN", players: 61 },
    { server: "KR", players: 23 },
    { server: "TW", players: 16 },
];

const CN_HEAVY = [
    { server: "EN", players: 31 },
    { server: "JP", players: 22 },
    { server: "CN", players: 174 },
    { server: "KR", players: 15 },
    { server: "TW", players: 8 },
];

export const Top250Split = () => (
    <div className="w-80">
        <ServerSplitCard shares={TOP_250} />
    </div>
);

export const CNDominated = () => (
    <div className="w-80">
        <ServerSplitCard shares={CN_HEAVY} />
    </div>
);

export const NoSnapshotYet = () => (
    <div className="w-80">
        <ServerSplitCard shares={[]} />
    </div>
);
