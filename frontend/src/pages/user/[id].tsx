import Head from "next/head";
import type { GetServerSideProps } from "next";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/shadcn/tabs";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { UserHeader } from "~/components/user/user-header";
import { CharactersGrid } from "~/components/user/characters-grid";
import { ItemsGrid } from "~/components/user/items-grid";
import { BaseView } from "~/components/user/base-view";
import type { StoredUser } from "~/types/api/impl/user";

interface UserPageProps {
    userData: StoredUser | null;
    error?: string;
}

export default function UserPage({ userData, error }: UserPageProps) {
    if (error || !userData || !userData.data) {
        return (
            <>
                <Head>
                    <title>User Not Found - myrtle.moe</title>
                    <meta name="description" content="User profile not found" />
                </Head>
                <div className="container mx-auto flex min-h-[50vh] items-center justify-center p-4">
                    <div className="text-center">
                        <h1 className="mb-4 text-4xl font-bold">User Not Found</h1>
                        <p className="text-muted-foreground">{error ?? "The requested user profile could not be found."}</p>
                    </div>
                </div>
            </>
        );
    }

    const data = userData.data;
    const nickName = data.status?.nickName ?? "Unknown";
    const possessive = nickName.endsWith("s") ? `${nickName}'` : `${nickName}'s`;

    return (
        <>
            <Head>
                <title>{possessive} Arknights Profile - myrtle.moe</title>
                <meta name="description" content={`View ${possessive} Arknights profile on myrtle.moe.`} />
            </Head>
            <div className="container mx-auto p-4">
                <UserHeader data={data} />

                <InView
                    variants={{
                        hidden: { opacity: 0, y: -20 },
                        visible: { opacity: 1, y: 0 },
                    }}
                    transition={{ duration: 0.5, ease: "easeOut", delay: 0.2 }}
                >
                    <Tabs defaultValue="characters" className="space-y-4">
                        <TabsList>
                            <TabsTrigger value="characters">Characters</TabsTrigger>
                            <TabsTrigger value="items">Items</TabsTrigger>
                            <TabsTrigger value="base">Base</TabsTrigger>
                        </TabsList>

                        <TabsContent value="characters" className="space-y-4">
                            <h2 className="text-2xl font-bold">Characters</h2>
                            <CharactersGrid data={data} />
                        </TabsContent>

                        <TabsContent value="items" className="space-y-4">
                            <h2 className="text-2xl font-bold">Items</h2>
                            <ItemsGrid data={data} />
                        </TabsContent>

                        <TabsContent value="base" className="space-y-4">
                            <h2 className="text-2xl font-bold">Base</h2>
                            <BaseView data={data} />
                        </TabsContent>
                    </Tabs>
                </InView>
            </div>
        </>
    );
}

export const getServerSideProps: GetServerSideProps<UserPageProps> = async (context) => {
    const { id } = context.params as { id: string };

    try {
        const { env } = await import("~/env");

        const backendUrl = new URL("/get-user", env.BACKEND_URL);
        backendUrl.searchParams.set("uid", id);

        const response = await fetch(backendUrl.toString(), {
            method: "GET",
            headers: {
                "Content-Type": "application/json",
            },
        });

        if (!response.ok) {
            if (response.status === 404) {
                return {
                    props: {
                        userData: null,
                        error: "User not found",
                    },
                };
            }
            const errorText = await response.text();
            console.error(`Backend get-user failed: ${response.status} - ${errorText}`);
            return {
                props: {
                    userData: null,
                    error: "Failed to fetch user data",
                },
            };
        }

        const userData: StoredUser = await response.json();

        if (!userData || !userData.data || !userData.data.status) {
            return {
                props: {
                    userData: null,
                    error: "Invalid user data received",
                },
            };
        }

        return {
            props: {
                userData,
            },
        };
    } catch (error) {
        console.error("Error fetching user data:", error);
        return {
            props: {
                userData: null,
                error: error instanceof Error ? error.message : "Failed to fetch user data",
            },
        };
    }
};
