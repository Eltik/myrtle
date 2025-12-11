import { motion } from "framer-motion";
import { LayoutGrid, List } from "lucide-react";
import type { NextPage } from "next";
import Head from "next/head";
import { useState } from "react";
import { Button } from "~/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/tabs";
import Base from "~/components/user/base";
import CharactersGrid from "~/components/user/charactersGrid";
import CharactersList from "~/components/user/charactersList";
import UserHeader from "~/components/user/header";
import Items from "~/components/user/items";
import { env } from "~/env";
import type { User } from "~/types/impl/api";
import type { ViewType } from "~/types/impl/frontend/impl/users";

const User: NextPage<Props> = ({ data }: { data: User }) => {
    const fadeIn = {
        hidden: { opacity: 0, y: -20 },
        visible: { opacity: 1, y: 0 },
    };

    const [currentView, setCurrentView] = useState<ViewType>("grid");

    const handleViewChange = (view: ViewType) => {
        setCurrentView(view);
    };

    if (!data) {
        return <></>;
    }

    return (
        <>
            <Head>
                <title>{data.status.nickName.endsWith("s") ? `${data.status.nickName}'` : `${data.status.nickName}'s`} Arknights Profile</title>
                <meta content={`View ${data.status.nickName.endsWith("s") ? `${data.status.nickName}'` : `${data.status.nickName}'s`} Arknights profile on myrtle.moe.`} name="description" />
                <link href="/favicon.ico" rel="icon" />
            </Head>
            <div className="container mx-auto p-4">
                <motion.div animate="visible" exit={"hidden"} initial="hidden" variants={fadeIn}>
                    <UserHeader data={data} />
                </motion.div>
                <motion.div animate="visible" exit={"hidden"} initial="hidden" variants={fadeIn}>
                    <Tabs className="space-y-4" defaultValue="characters">
                        <TabsList>
                            <TabsTrigger value="characters">Characters</TabsTrigger>
                            <TabsTrigger value="items">Items</TabsTrigger>
                            <TabsTrigger value="base">Base</TabsTrigger>
                        </TabsList>
                        <TabsContent className="space-y-4" value="characters">
                            <div className="flex flex-row gap-4">
                                <h2 className="font-bold text-2xl">Characters</h2>
                                <div className="flex rounded-md shadow-sm" role="group">
                                    <Button aria-label="Grid view" className="rounded-r-none rounded-l-md border-r-0" onClick={() => handleViewChange("grid")} variant={currentView === "grid" ? "default" : "outline"}>
                                        <LayoutGrid className="h-4 w-4" />
                                    </Button>
                                    <Button aria-label="List view" className="rounded-r-md rounded-l-none border-l-0" onClick={() => handleViewChange("list")} variant={currentView === "list" ? "default" : "outline"}>
                                        <List className="h-4 w-4" />
                                    </Button>
                                </div>
                            </div>
                            {currentView === "grid" ? <CharactersGrid data={data} /> : <CharactersList data={data} />}
                        </TabsContent>
                        <TabsContent className="space-y-4" value="items">
                            <h2 className="font-bold text-2xl">Items</h2>
                            <Items data={data} />
                        </TabsContent>
                        <TabsContent className="space-y-4" value="base">
                            <h2 className="font-bold text-2xl">Base</h2>
                            <Base data={data} />
                        </TabsContent>
                    </Tabs>
                </motion.div>
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

    const user = data[0]?.data;

    // Compress the data
    const compressedData = {
        status: user?.status,
        troop: {
            chars: user?.troop.chars,
        },
        inventory: user?.inventory,
        building: user?.building,
    };

    return {
        props: {
            data: user ? compressedData : null,
        },
    };
};

export default User;

interface Props {
    data: User;
}
