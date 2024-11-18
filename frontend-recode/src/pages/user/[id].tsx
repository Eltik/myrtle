import { Clipboard, ArrowUpDown } from "lucide-react";
import type { NextPage } from "next";
import Head from "next/head";
import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/avatar";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card";
import { ScrollArea } from "~/components/ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/tabs";
import CharacterCard from "~/components/user/character-card";
import Stat from "~/components/user/stat";
import { env } from "~/env";
import type { User } from "~/types/impl/api";
import type { ColumnDef } from "@tanstack/react-table";
import type { Item } from "~/types/impl/api/static/material";
import { DataTable } from "~/components/user/data-table";
import Image from "next/image";
import { toast } from "~/hooks/use-toast";
import { Button } from "~/components/ui/button";
import { Progress } from "~/components/ui/progress";
import { Badge } from "~/components/ui/badge";
import { getAvatarSkinId } from "~/helper";

const User: NextPage<Props> = ({ data }: { data: User }) => {
    if (!data) {
        return <></>;
    }

    const columns: ColumnDef<
        | (Item & {
              amount: number;
          })
        | null
    >[] = [
        {
            accessorKey: "icon",
            header: () => "Icon",
            cell: ({ row }) => {
                return <Image src={`https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/item/${String(row.getValue("icon"))}.png`} width={32} height={32} alt={"Item icon"} />;
            },
        },
        {
            accessorKey: "name",
            header: ({ column }) => {
                return (
                    <Button variant="ghost" onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}>
                        Name
                        <ArrowUpDown className="ml-2 h-4 w-4" />
                    </Button>
                );
            },
        },
        {
            accessorKey: "amount",
            header: ({ column }) => {
                return (
                    <Button variant="ghost" onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}>
                        Amount
                        <ArrowUpDown className="ml-2 h-4 w-4" />
                    </Button>
                );
            },
        },
    ];

    return (
        <>
            <Head>
                <title>{data.status.nickName.endsWith("s") ? `${data.status.nickName}'` : `${data.status.nickName}'s`} Arknights Profile</title>
                <meta name="description" content={`View ${data.status.nickName.endsWith("s") ? `${data.status.nickName}'` : `${data.status.nickName}'s`} Arknights profile on myrtle.moe.`} />
                <link rel="icon" href="/favicon.ico" />
            </Head>
            <div className="container mx-auto p-4">
                <Card className="mx-auto mb-8 mt-5">
                    <CardHeader>
                        <div className="flex items-center space-x-4">
                            <Avatar className="h-20 w-20">
                                <AvatarImage src={getAvatarSkinId(data)} alt="@shadcn" />
                                <AvatarFallback>{data.status?.nickName?.slice(0, 1) ?? "E"}</AvatarFallback>
                            </Avatar>
                            <div>
                                <CardTitle className="text-2xl">
                                    <div className="flex flex-row gap-4">
                                        <div>
                                            {data.status.nickName}
                                            <span className="text-muted-foreground">#{data.status.nickNumber}</span>
                                        </div>
                                        <div
                                            className="hidden cursor-pointer rounded-md border p-2 transition-all duration-150 hover:bg-secondary md:block"
                                            onClick={() => {
                                                void navigator.clipboard.writeText(`${data.status.nickName}#${data.status.nickNumber}`);
                                                toast({
                                                    title: "Success!",
                                                    description: "Copied username to clipboard.",
                                                });
                                            }}
                                        >
                                            <Clipboard size={15} />
                                        </div>
                                    </div>
                                </CardTitle>
                                <CardDescription>
                                    <div className="flex flex-col">
                                        <span>Level {data.status.level}</span>
                                        <span>{data.status.resume}</span>
                                    </div>
                                </CardDescription>
                            </div>
                        </div>
                    </CardHeader>
                    <CardContent>
                        <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
                            <Stat label="LMD" value={data.status.gold} />
                            <Stat label="Orundum" value={data.status.diamondShard} />
                            <Stat label="Originium" value={data.status.payDiamond + data.status.freeDiamond} />
                            <Stat label="Friend Limit" value={data.status.friendNumLimit} />
                        </div>
                    </CardContent>
                </Card>
                <Tabs defaultValue="characters" className="space-y-4">
                    <TabsList>
                        <TabsTrigger value="characters">Characters</TabsTrigger>
                        <TabsTrigger value="items">Items</TabsTrigger>
                        <TabsTrigger value="base">Base</TabsTrigger>
                    </TabsList>
                    <TabsContent value="characters" className="space-y-4">
                        <h2 className="text-2xl font-bold">Characters</h2>
                        {/*
                        <ScrollArea className="h-[100vh] rounded-md border p-4 lg:h-96">
                            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
                                {Object.values(data.troop.chars).map((char) => (
                                    <CharacterCard key={char.charId} data={char} />
                                ))}
                            </div>
                        </ScrollArea>
                        */}
                        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
                            {Object.values(data.troop.chars).map((char) => (
                                <CharacterCard key={char.charId} data={char} />
                            ))}
                        </div>
                    </TabsContent>
                    <TabsContent value="items" className="space-y-4">
                        <h2 className="text-2xl font-bold">Items</h2>
                        <DataTable
                            columns={columns}
                            data={Object.values(data.inventory)
                                .map((item) => {
                                    if (!item.iconId) return null;
                                    return {
                                        ...item,
                                        icon: item.iconId,
                                    };
                                })
                                .filter(Boolean)}
                        />
                    </TabsContent>
                    <TabsContent value="base" className="space-y-4">
                        <h2 className="text-2xl font-bold">Base</h2>
                        <ScrollArea className="h-[100vh] rounded-md border p-4 lg:h-96">
                            <Card className="mb-6">
                                <CardHeader>
                                    <CardTitle className="text-xl">Base Overview</CardTitle>
                                </CardHeader>
                                <CardContent>
                                    <div className="space-y-2">
                                        <div>
                                            <p className="text-sm text-muted-foreground">Drones</p>
                                            <Progress value={(data.building.status.labor.value / data.building.status.labor.maxValue) * 100} className="w-64" />
                                        </div>
                                        <div>
                                            <p className="text-sm text-muted-foreground">Trading Posts</p>
                                            <div className="flex flex-wrap gap-2">
                                                {Object.values(data.building.rooms.TRADING).map((post, index) => (
                                                    <Badge key={index} variant={post.state === 1 ? "default" : "secondary"}>
                                                        {post.state === 1 ? "Active" : "Inactive"}
                                                    </Badge>
                                                ))}
                                            </div>
                                        </div>
                                        <div>
                                            <p className="text-sm text-muted-foreground">Factories</p>
                                            <div className="flex flex-wrap gap-2">
                                                {Object.values(data.building.rooms.MANUFACTURE).map((factory, index) => (
                                                    <Badge key={index} variant={factory.state === 1 ? "default" : "secondary"}>
                                                        {factory.state === 1 ? "Active" : "Inactive"}
                                                    </Badge>
                                                ))}
                                            </div>
                                        </div>
                                    </div>
                                </CardContent>
                            </Card>
                        </ScrollArea>
                    </TabsContent>
                </Tabs>
            </div>
        </>
    );
};

export const getServerSideProps = async ({ query }: { query: { id: string } }) => {
    const { id } = query;
    const server = "en";

    const data = (await (await fetch(`${env.BACKEND_URL}/player/${id}/${server}`)).json()) as
        | {
              id: string;
              uid: string;
              server: string;
              data: User;
              created_at: string;
          }[]
        | null;
    if (!data) return { props: { notFound: true } };

    return {
        props: {
            data: data[0]?.data,
        },
    };
};

export default User;

interface Props {
    data: User;
}
