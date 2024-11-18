import { Clipboard } from "lucide-react";
import type { NextPage } from "next";
import Head from "next/head";
import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/avatar";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/tabs";
import Stat from "~/components/user/components/stat";
import { env } from "~/env";
import type { User } from "~/types/impl/api";
import { toast } from "~/hooks/use-toast";
import { getAvatarSkinId } from "~/helper";
import Characters from "~/components/user/characters";
import Items from "~/components/user/items";
import Base from "~/components/user/base";

const User: NextPage<Props> = ({ data }: { data: User }) => {
    if (!data) {
        return <></>;
    }

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
                        <Characters data={data} />
                    </TabsContent>
                    <TabsContent value="items" className="space-y-4">
                        <Items data={data} />
                    </TabsContent>
                    <TabsContent value="base" className="space-y-4">
                        <Base data={data} />
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
