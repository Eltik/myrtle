import type { GetServerSideProps } from "next";
import { SEO } from "~/components/seo";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/shadcn/tabs";
import { CharactersGrid } from "~/components/user/characters-grid";
import { ItemsGrid } from "~/components/user/items-grid";
import { ScoreView } from "~/components/user/score-view";
import { UserHeader } from "~/components/user/user-header";
import type { StoredUser } from "~/types/api/impl/user";

interface UserPageProps {
    userData: StoredUser | null;
    userId: string;
    error?: string;
}

export default function UserPage({ userData, userId, error }: UserPageProps) {
    if (error || !userData || !userData.data) {
        return (
            <>
                <SEO description="User profile not found on myrtle.moe." noIndex path={`/user/${userId}`} title="User Not Found" />
                <div className="container mx-auto flex min-h-[50vh] items-center justify-center p-4">
                    <div className="text-center">
                        <h1 className="mb-4 font-bold text-4xl">User Not Found</h1>
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
            <SEO description={`View ${possessive} Arknights profile on myrtle.moe. See their operator roster, inventory, and account score.`} keywords={[nickName, "Arknights profile", "player profile", "operator collection"]} path={`/user/${userId}`} title={`${possessive} Arknights Profile`} type="profile" />
            <div className="container mx-auto p-4">
                <UserHeader data={data} />

                <InView
                    transition={{ duration: 0.5, ease: "easeOut", delay: 0.2 }}
                    variants={{
                        hidden: { opacity: 0, y: -20 },
                        visible: { opacity: 1, y: 0 },
                    }}
                >
                    <Tabs className="space-y-4" defaultValue="characters">
                        <TabsList>
                            <TabsTrigger value="characters">Characters</TabsTrigger>
                            <TabsTrigger value="items">Items</TabsTrigger>
                            <TabsTrigger value="score">Score</TabsTrigger>
                        </TabsList>

                        <TabsContent className="space-y-4" value="characters">
                            <h2 className="font-bold text-2xl">Characters</h2>
                            <CharactersGrid data={data} />
                        </TabsContent>

                        <TabsContent className="space-y-4" value="items">
                            <h2 className="font-bold text-2xl">Items</h2>
                            <ItemsGrid data={data} />
                        </TabsContent>

                        <TabsContent className="space-y-4" value="score">
                            <h2 className="font-bold text-2xl">Score & Grade</h2>
                            <ScoreView scoreData={userData.score} />
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

        const backendURL = new URL("/get-user", env.BACKEND_URL);
        backendURL.searchParams.set("uid", id);

        const response = await fetch(backendURL.toString(), {
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
                        userId: id,
                        error: "User not found",
                    },
                };
            }
            const errorText = await response.text();
            console.error(`Backend get-user failed: ${response.status} - ${errorText}`);
            return {
                props: {
                    userData: null,
                    userId: id,
                    error: "Failed to fetch user data",
                },
            };
        }

        const userData: StoredUser = await response.json();

        if (!userData || !userData.data || !userData.data.status) {
            return {
                props: {
                    userData: null,
                    userId: id,
                    error: "Invalid user data received",
                },
            };
        }

        return {
            props: {
                userData,
                userId: id,
            },
        };
    } catch (error) {
        console.error("Error fetching user data:", error);
        return {
            props: {
                userData: null,
                userId: id,
                error: error instanceof Error ? error.message : "Failed to fetch user data",
            },
        };
    }
};
