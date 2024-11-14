import type { NextPage } from "next";
import Navbar from "~/components/navbar";
import { PlayerProfile } from "~/components/player-profile";
import { env } from "~/env.mjs";
import ClientOnly from "~/lib/ClientOnly";
import type { PlayerData } from "~/types/types";

const User: NextPage<Props> = ({ data }: { data: PlayerData }) => {
    return (
        <>
            <main>
                <Navbar />
                <ClientOnly>
                    <div className="flex flex-1 flex-col gap-6 p-4 md:p-10">
                        {data ? (
                            <PlayerProfile data={data} />
                        ) : (
                            <div className="flex flex-col gap-6">
                                <h1 className="text-4xl font-bold">User not found!</h1>
                                <p className="text-lg">The user you are looking for does not exist in the database. You can ask the user to login to Myrtle to be added!</p>
                            </div>
                        )}
                    </div>
                </ClientOnly>
            </main>
        </>
    );
};

export const getServerSideProps = async ({ query }: { query: { id: string } }) => {
    const { id } = query;
    const server = "en";

    const data = (await (await fetch(`${env.BACKEND_URL}/player/${id}/${server}`)).json()) as {
        id: string;
        uid: string;
        server: string;
        data: PlayerData;
        created_at: string;
    }[] | null;
    if (!data) return { props: { notFound: true } };

    return {
        props: {
            data: data[0]?.data,
        },
    };
};

export default User;

interface Props {
    data: PlayerData;
}
