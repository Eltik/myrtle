import { LayoutGrid, List } from "lucide-react";
import type { NextPage } from "next";
import Head from "next/head";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/tabs";
import { env } from "~/env";
import type { User } from "~/types/impl/api";
import CharactersGrid from "~/components/user/charactersGrid";
import Items from "~/components/user/items";
import Base from "~/components/user/base";
import type { ViewType } from "~/types/impl/frontend/impl/users";
import { useState } from "react";
import { Button } from "~/components/ui/button";
import CharactersList from "~/components/user/charactersList";
import UserHeader from "~/components/user/header";
import { motion } from "framer-motion";

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
                <meta name="description" content={`View ${data.status.nickName.endsWith("s") ? `${data.status.nickName}'` : `${data.status.nickName}'s`} Arknights profile on myrtle.moe.`} />
                <link rel="icon" href="/favicon.ico" />
            </Head>
            <div className="container mx-auto p-4">
                <motion.div
                    initial="hidden"
                    animate="visible"
                    exit={"hidden"}
                    variants={{
                        hidden: { opacity: 0 },
                        visible: {
                            opacity: 1,
                            transition: { staggerChildren: 0.2 },
                        },
                    }}
                >
                    <UserHeader data={data} />
                </motion.div>
                <motion.div
                    initial="hidden"
                    animate="visible"
                    exit={"hidden"}
                    variants={{
                        hidden: { opacity: 0 },
                        visible: {
                            opacity: 1,
                            transition: { staggerChildren: 0.2 },
                        },
                    }}
                >
                    <Tabs defaultValue="characters" className="space-y-4">
                        <TabsList>
                            <TabsTrigger value="characters">Characters</TabsTrigger>
                            <TabsTrigger value="items">Items</TabsTrigger>
                            <TabsTrigger value="base">Base</TabsTrigger>
                        </TabsList>
                        <TabsContent value="characters" className="space-y-4">
                            <div className="flex flex-row gap-4">
                                <h2 className="text-2xl font-bold">Characters</h2>
                                <div className="flex rounded-md shadow-sm" role="group">
                                    <Button variant={currentView === "grid" ? "default" : "outline"} className="rounded-l-md rounded-r-none border-r-0" onClick={() => handleViewChange("grid")} aria-label="Grid view">
                                        <LayoutGrid className="h-4 w-4" />
                                    </Button>
                                    <Button variant={currentView === "list" ? "default" : "outline"} className="rounded-l-none rounded-r-md border-l-0" onClick={() => handleViewChange("list")} aria-label="List view">
                                        <List className="h-4 w-4" />
                                    </Button>
                                </div>
                            </div>
                            {currentView === "grid" ? <CharactersGrid data={data} /> : <CharactersList data={data} />}
                        </TabsContent>
                        <TabsContent value="items" className="space-y-4">
                            <h2 className="text-2xl font-bold">Items</h2>
                            <Items data={data} />
                        </TabsContent>
                        <TabsContent value="base" className="space-y-4">
                            <h2 className="text-2xl font-bold">Base</h2>
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
