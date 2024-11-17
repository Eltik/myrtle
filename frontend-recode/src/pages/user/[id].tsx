import { Clipboard } from "lucide-react";
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
            header: () => <div className="text-right">Icon</div>,
            cell: ({ row }) => {
                return <Image src={`https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/item/${String(row.getValue("icon"))}.png`} width={32} height={32} alt={"Item icon"} />;
            },
        },
        {
            accessorKey: "name",
            header: () => <div className="text-right">Name</div>,
        },
        {
            accessorKey: "amount",
            header: () => <div className="text-right">Amount</div>,
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
                <Card className="mb-8">
                    <CardHeader>
                        <div className="flex items-center space-x-4">
                            <Avatar className="h-20 w-20">
                                <AvatarImage
                                    src={
                                        data.status?.avatarId
                                            ? data.status.avatar.type === "ASSISTANT"
                                                ? `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/avatar/${encodeURIComponent(
                                                      (Object.values(data.troop.chars).find((item) => item.skin === data.status.avatar.id)?.charId ?? "").includes("@")
                                                          ? (Object.values(data.troop.chars)
                                                                .find((item) => item.skin === data.status.avatar.id)
                                                                ?.charId?.replaceAll("@", "_") ?? "")
                                                          : (Object.values(data.troop.chars)
                                                                .find((item) => item.skin === data.status.avatar.id)
                                                                ?.charId?.replaceAll("#", "_") ?? ""),
                                                  )}.png`
                                                : ""
                                            : "https://static.wikia.nocookie.net/mrfz/images/4/46/Symbol_profile.png/revision/latest?cb=20220418145951"
                                    }
                                    alt="@shadcn"
                                />
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
                                            className="cursor-pointer rounded-md border p-2 transition-all duration-150 hover:bg-secondary"
                                            onClick={() => {
                                                navigator.clipboard.writeText(`${data.status.nickName}#${data.status.nickNumber}`);
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
                                <CardDescription>Level {data.status.level}</CardDescription>
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
                        <TabsTrigger value="stats">Stats</TabsTrigger>
                    </TabsList>
                    <TabsContent value="characters" className="space-y-4">
                        <h2 className="text-2xl font-bold">Characters</h2>
                        <ScrollArea className="h-[400px] rounded-md border p-4">
                            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
                                {Object.values(data.troop.chars).map((char) => (
                                    <CharacterCard key={char.charId} data={char} />
                                ))}
                            </div>
                        </ScrollArea>
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
