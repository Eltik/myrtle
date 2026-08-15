import { AdminStatTile } from "frontend";

// Sparkline polylines are plotted in the tile's own 200x28 viewBox.
const usersSpark = "0,24 20,22 40,23 60,18 80,19 100,14 120,16 140,10 160,12 180,6 200,4";
const rostersSpark = "0,20 20,21 40,17 60,18 80,13 100,15 120,12 140,13 160,8 180,10 200,7";
const viewsSpark = "0,7 20,9 40,8 60,13 80,11 100,16 120,15 140,12 160,18 180,20 200,23";

export function DashboardRow() {
    return (
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-4 sm:gap-3.5">
            <AdminStatTile label="Total users" value="18.4K" color="var(--chart-1)" />
            <AdminStatTile label="Rosters synced" value="12.9K" color="var(--chart-2)" />
            <AdminStatTile label="Tier lists · active" value="37" unit="of 214" color="var(--chart-2)" />
            <AdminStatTile label="Tier-list placements" value="41.6K" color="var(--chart-4)" />
        </div>
    );
}

export function WithTrendDeltas() {
    return (
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 sm:gap-3.5">
            <AdminStatTile label="New doctors · 7d" value="1,284" delta="+11.2% vs prior week" deltaDir="up" color="var(--chart-1)" />
            <AdminStatTile label="Cache hit rate" value="98.1" unit="%" delta="+0.6 pts" deltaDir="up" color="var(--chart-2)" />
            <AdminStatTile label="Rosters synced · 7d" value="908" unit="rosters" delta="-6.4% vs prior week" deltaDir="down" color="var(--chart-4)" />
        </div>
    );
}

export function WithSparklines() {
    return (
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 sm:gap-3.5">
            <AdminStatTile label="Total users" value="18,412" delta="+624 this week" deltaDir="up" color="var(--chart-1)" spark={usersSpark} />
            <AdminStatTile label="Rosters synced" value="12,908" delta="+1,033 this week" deltaDir="up" color="var(--chart-2)" spark={rostersSpark} />
            <AdminStatTile label="Tier-list views · 24h" value="6,204" delta="-12.8% vs yesterday" deltaDir="down" color="var(--chart-4)" spark={viewsSpark} />
        </div>
    );
}

export function GameDataCounts() {
    return (
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-4 sm:gap-3.5">
            <AdminStatTile label="Operators" value="438" color="var(--primary)" />
            <AdminStatTile label="Skills" value="1,207" color="var(--chart-1)" />
            <AdminStatTile label="Stages" value="4,812" color="var(--chart-2)" />
            <AdminStatTile label="Enemies" value="1,163" color="var(--chart-4)" />
        </div>
    );
}
