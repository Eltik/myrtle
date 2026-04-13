import type { GetServerSideProps } from "next";
import { SEO } from "~/components/seo";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/shadcn/tabs";
import { CharactersGrid } from "~/components/user/characters-grid";
import { ScoreView } from "~/components/user/score-view";
import { StatsView } from "~/components/user/stats-view";
import { UserHeader } from "~/components/user/user-header";
import { env } from "~/env";
import type { UserProfile } from "~/types/api/impl/user";

interface UserPageProps {
    profile: UserProfile | null;
    userId: string;
    baseUrl: string;
    error?: string;
}

export default function UserPage({ profile, userId, baseUrl, error }: UserPageProps) {
    if (error || !profile) {
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

    const nickName = profile.nickname ?? "Unknown";
    const possessive = nickName.endsWith("s") ? `${nickName}'` : `${nickName}'s`;

    return (
        <>
            <SEO
                description={`View ${possessive} Arknights profile on myrtle.moe. See their operator roster, inventory, and account score.`}
                image={`${baseUrl}/api/og/user?id=${userId}`}
                keywords={[nickName, "Arknights profile", "player profile", "operator collection"]}
                path={`/user/${userId}`}
                title={`${possessive} Arknights Profile`}
                type="profile"
            />
            <div className="container mx-auto p-4">
                <UserHeader profile={profile} />

                <InView
                    once
                    transition={{ duration: 0.5, ease: "easeOut", delay: 0.2 }}
                    variants={{
                        hidden: { opacity: 0, y: -20 },
                        visible: { opacity: 1, y: 0 },
                    }}
                >
                    <Tabs className="space-y-4" defaultValue="characters">
                        <TabsList>
                            <TabsTrigger value="characters">Characters</TabsTrigger>
                            <TabsTrigger value="stats">Stats</TabsTrigger>
                            <TabsTrigger value="score">Score</TabsTrigger>
                        </TabsList>

                        <TabsContent className="space-y-4" value="characters">
                            <h2 className="font-bold text-2xl">Characters</h2>
                            <CharactersGrid userId={userId} />
                        </TabsContent>

                        <TabsContent className="space-y-4" value="stats">
                            <h2 className="font-bold text-2xl">Account Stats</h2>
                            <StatsView userId={userId} />
                        </TabsContent>

                        <TabsContent className="space-y-4" value="score">
                            <h2 className="font-bold text-2xl">Score & Grade</h2>
                            <ScoreView userId={userId} />
                        </TabsContent>
                    </Tabs>
                </InView>
            </div>
        </>
    );
}

export const getServerSideProps: GetServerSideProps<UserPageProps> = async (context) => {
    const { id } = context.params as { id: string };
    const { req } = context;

    // Build base URL from request headers
    const protocol = req.headers["x-forwarded-proto"] || "https";
    const host = req.headers.host || "myrtle.moe";
    const baseUrl = env.NODE_ENV === "production" ? "https://myrtle.moe" : `${protocol}://${host}`;

    try {
        const { env } = await import("~/env");
        const { backendFetch } = await import("~/lib/backend-fetch");

        const response = await backendFetch(`/get-user?uid=${encodeURIComponent(id)}`);

        if (!response.ok) {
            if (response.status === 404) {
                return {
                    props: {
                        profile: null,
                        userId: id,
                        baseUrl,
                        error: "User not found",
                    },
                };
            }
            const errorText = await response.text();
            console.error(`Backend get-user failed: ${response.status} - ${errorText}`);
            return {
                props: {
                    profile: null,
                    userId: id,
                    baseUrl,
                    error: "Failed to fetch user data",
                },
            };
        }

        const profile: UserProfile = await response.json();

        if (!profile || !profile.uid) {
            return {
                props: {
                    profile: null,
                    userId: id,
                    baseUrl,
                    error: "Invalid user data received",
                },
            };
        }

        return {
            props: {
                profile,
                userId: id,
                baseUrl,
            },
        };
    } catch (error) {
        console.error("Error fetching user data:", error);
        return {
            props: {
                profile: null,
                userId: id,
                baseUrl,
                error: error instanceof Error ? error.message : "Failed to fetch user data",
            },
        };
    }
};
