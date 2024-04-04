import { ArrowUpIcon, GlobeIcon, StarIcon, TrophyIcon } from "lucide-react";
import type { NextPage } from "next";
import Link from "next/link";
import Navbar from "~/components/navbar";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "~/components/ui/table";
import { env } from "~/env.mjs";
import ClientOnly from "~/lib/ClientOnly";
import type { PlayerData } from "~/types/types";

const Leaderboard: NextPage<Props> = ({ data, type }: { data: PlayerData[]; type: string }) => {
    return (
        <>
            <main>
                <Navbar />
                <ClientOnly>
                    <div className="flex min-h-[calc(100vh_-_theme(spacing.16))] flex-1 flex-col gap-4 p-4 md:gap-8 md:p-10">
                        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                            <Card>
                                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                                    <CardTitle className="text-sm font-medium">Player Rank</CardTitle>
                                    <TrophyIcon className="h-4 w-4 text-gray-500 dark:text-gray-400" />
                                </CardHeader>
                                <CardContent>
                                    <div className="text-2xl font-bold">1st</div>
                                    <p className="text-xs text-gray-500 dark:text-gray-400">Player 1</p>
                                </CardContent>
                            </Card>
                            <Card>
                                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                                    <CardTitle className="text-sm font-medium">Player Score</CardTitle>
                                    <StarIcon className="h-4 w-4 text-gray-500 dark:text-gray-400" />
                                </CardHeader>
                                <CardContent>
                                    <div className="text-2xl font-bold">1000</div>
                                    <p className="text-xs text-gray-500 dark:text-gray-400">Player 1</p>
                                </CardContent>
                            </Card>
                            <Card>
                                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                                    <CardTitle className="text-sm font-medium">Player Level</CardTitle>
                                    <ArrowUpIcon className="h-4 w-4 text-gray-500 dark:text-gray-400" />
                                </CardHeader>
                                <CardContent>
                                    <div className="text-2xl font-bold">10</div>
                                    <p className="text-xs text-gray-500 dark:text-gray-400">Player 1</p>
                                </CardContent>
                            </Card>
                            <Card>
                                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                                    <CardTitle className="text-sm font-medium">Player Region</CardTitle>
                                    <GlobeIcon className="h-4 w-4 text-gray-500 dark:text-gray-400" />
                                </CardHeader>
                                <CardContent>
                                    <div className="text-2xl font-bold">Asia</div>
                                    <p className="text-xs text-gray-500 dark:text-gray-400">Player 1</p>
                                </CardContent>
                            </Card>
                        </div>
                        <div>
                            <Card>
                                <Table>
                                    <TableHeader>
                                        <TableRow>
                                            <TableHead className="w-[100px]">Rank</TableHead>
                                            <TableHead>Player</TableHead>
                                            <TableHead>{type === "trust" ? "Average Trust" : type === "level" ? "Total Exp." : "Unknown"}</TableHead>
                                            <TableHead>Level</TableHead>
                                            <TableHead>Start Time</TableHead>
                                        </TableRow>
                                    </TableHeader>
                                    <TableBody>
                                        {data.map((player, index) => (
                                            <TableRow key={`leaderboard-${index}`}>
                                                <TableCell className="font-medium">{index + 1}</TableCell>
                                                <TableCell>
                                                    <Link href={`/user/${player.status.uid}`}>{player.status.nickName}</Link>
                                                </TableCell>
                                                <TableCell>{type === "trust" ? Math.round((player as PlayerData & { avg_favorPoint: number }).avg_favorPoint) : type === "level" ? player.status.exp : "Unknown"}</TableCell>
                                                <TableCell>{player.status.level}</TableCell>
                                                <TableCell>{new Date(player.status.registerTs * 1000).toDateString()}</TableCell>
                                            </TableRow>
                                        ))}
                                    </TableBody>
                                </Table>
                            </Card>
                        </div>
                    </div>
                </ClientOnly>
            </main>
        </>
    );
};

export const getServerSideProps = async ({ query }: { query: { type: string; sort: string } }) => {
    const { type, sort } = query;
    const server = "en";

    const data = (await (await fetch(`${env.BACKEND_URL}/leaderboard?server=${server}&type=${type}&sort=${sort}&fields=[status,avg_favorPoint]`)).json()) as PlayerData[];

    return {
        props: {
            type,
            data,
        },
    };
};

export default Leaderboard;

interface Props {
    type: string;
    data: PlayerData[];
}
