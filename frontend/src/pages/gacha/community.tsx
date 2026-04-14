import { BarChart3, Sparkles, Star, Users } from "lucide-react";
import type { GetServerSideProps, NextPage } from "next";
import { SEO } from "~/components/seo";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import type { GachaGlobalStats } from "~/types/api/impl/gacha";

interface GlobalGachaStatsPageProps {
    stats: GachaGlobalStats | null;
    error?: string;
}

const GlobalGachaStatsPage: NextPage<GlobalGachaStatsPageProps> = ({ stats, error }) => {
    // Error state
    if (error || !stats) {
        return (
            <>
                <SEO description="Community-wide Arknights gacha statistics and pull rates." path="/gacha/community" title="Community Stats" />
                <div className="container mx-auto flex min-h-[50vh] items-center justify-center p-4">
                    <div className="text-center">
                        <h1 className="mb-4 font-bold text-4xl">Statistics Unavailable</h1>
                        <p className="text-muted-foreground">{error ?? "Unable to load global statistics at this time."}</p>
                    </div>
                </div>
            </>
        );
    }

    const sixStarRate = stats.totalPulls > 0 ? ((stats.sixStarRate) * 100).toFixed(2) : "0.00";
    const fiveStarRate = stats.totalPulls > 0 ? ((stats.fiveStarRate) * 100).toFixed(2) : "0.00";

    return (
        <>
            <SEO
                description={`Community gacha statistics from ${stats.totalUsers.toLocaleString()} players and ${stats.totalPulls.toLocaleString()} pulls.`}
                keywords={["gacha statistics", "pull rates", "6-star rate", "Arknights gacha"]}
                path="/gacha/community"
                title="Community Stats"
            />
            <div className="container mx-auto space-y-8 p-4 py-8">
                {/* Header */}
                <div className="space-y-2">
                    <h1 className="font-bold text-3xl md:text-4xl">Community Gacha Statistics</h1>
                    <p className="text-muted-foreground">
                        Aggregated pull data from {stats.totalUsers.toLocaleString()} contributing players
                    </p>
                </div>

                {/* Main Statistics Cards */}
                <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                    <Card>
                        <CardContent className="flex items-center gap-4 p-6">
                            <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-primary/10">
                                <BarChart3 className="h-6 w-6 text-primary" />
                            </div>
                            <div>
                                <p className="text-muted-foreground text-sm">Total Pulls</p>
                                <p className="font-bold text-2xl">{stats.totalPulls.toLocaleString()}</p>
                            </div>
                        </CardContent>
                    </Card>

                    <Card>
                        <CardContent className="flex items-center gap-4 p-6">
                            <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-primary/10">
                                <Users className="h-6 w-6 text-primary" />
                            </div>
                            <div>
                                <p className="text-muted-foreground text-sm">Contributing Players</p>
                                <p className="font-bold text-2xl">{stats.totalUsers.toLocaleString()}</p>
                            </div>
                        </CardContent>
                    </Card>

                    <Card>
                        <CardContent className="flex items-center gap-4 p-6">
                            <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-orange-500/10">
                                <Sparkles className="h-6 w-6 text-orange-500" />
                            </div>
                            <div>
                                <p className="text-muted-foreground text-sm">6-Star Rate</p>
                                <p className="font-bold text-2xl text-orange-500">{sixStarRate}%</p>
                            </div>
                        </CardContent>
                    </Card>

                    <Card>
                        <CardContent className="flex items-center gap-4 p-6">
                            <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-yellow-500/10">
                                <Star className="h-6 w-6 text-yellow-500" />
                            </div>
                            <div>
                                <p className="text-muted-foreground text-sm">5-Star Rate</p>
                                <p className="font-bold text-2xl text-yellow-500">{fiveStarRate}%</p>
                            </div>
                        </CardContent>
                    </Card>
                </div>

                {/* Rate Comparison */}
                <Card>
                    <CardHeader>
                        <CardTitle>Pull Rate Analysis</CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        <RateBar actual={stats.sixStarRate} color="#ff6e40" expected={0.02} label="6-Star" />
                        <RateBar actual={stats.fiveStarRate} color="#ffd740" expected={0.08} label="5-Star" />
                    </CardContent>
                </Card>

                {/* Data Source Notice */}
                <div className="text-center text-muted-foreground text-sm">
                    <p>Statistics are based on anonymized pull data from participating users.</p>
                    <p>Data is refreshed periodically and may not reflect real-time values.</p>
                </div>
            </div>
        </>
    );
};

function RateBar({ label, actual, expected, color }: { label: string; actual: number; expected: number; color: string }) {
    const actualPct = (actual * 100).toFixed(2);
    const expectedPct = (expected * 100).toFixed(2);
    const barWidth = Math.min((actual / expected) * 50, 100); // Scale relative to expected

    return (
        <div className="space-y-1">
            <div className="flex items-center justify-between text-sm">
                <span className="font-medium">{label}</span>
                <span className="font-mono">
                    {actualPct}% <span className="text-muted-foreground">(expected: {expectedPct}%)</span>
                </span>
            </div>
            <div className="h-3 w-full rounded-full bg-secondary">
                <div className="h-full rounded-full transition-all duration-500" style={{ width: `${barWidth}%`, backgroundColor: color }} />
            </div>
        </div>
    );
}

export const getServerSideProps: GetServerSideProps<GlobalGachaStatsPageProps> = async () => {
    try {
        const { env } = await import("~/env");
        const { backendFetch } = await import("~/lib/backend-fetch");

        const response = await backendFetch("/gacha/stats");

        if (!response.ok) {
            console.error(`Global gacha stats fetch failed: ${response.status}`);
            return {
                props: {
                    stats: null,
                    error: "Failed to fetch statistics",
                },
            };
        }

        const stats: GachaGlobalStats = await response.json();

        return {
            props: {
                stats,
            },
        };
    } catch (error) {
        console.error("Error fetching global gacha stats:", error);
        return {
            props: {
                stats: null,
                error: error instanceof Error ? error.message : "An unexpected error occurred",
            },
        };
    }
};

export default GlobalGachaStatsPage;
