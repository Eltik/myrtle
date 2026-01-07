import type { GetServerSideProps, NextPage } from "next";
import Head from "next/head";
import { env } from "~/env";
import type { Stage, StagesResponse } from "~/types/api/impl/stage";
import type { Zone, ZonesResponse } from "~/types/api/impl/zone";

interface Props {
    zones: Zone[];
    stages: Stage[];
}

const RandomizerPage: NextPage<Props> = ({ zones, stages }) => {
    return (
        <>
            <Head>
                <title>Stage Randomizer | myrtle.moe</title>
                <meta content="Randomize Arknights stages for challenge runs and squad randomization." name="description" />
            </Head>
            <main className="container mx-auto px-4 py-8">
                <h1 className="mb-4 font-bold text-2xl">Stage Randomizer</h1>
                <p className="mb-4 text-muted-foreground">
                    Zones loaded: {zones.length} | Stages loaded: {stages.length}
                </p>
                {/* UI to be implemented later */}
            </main>
        </>
    );
};

export const getServerSideProps: GetServerSideProps<Props> = async () => {
    const backendURL = env.BACKEND_URL;

    try {
        // Fetch zones (permanent content only: MAINLINE, SIDESTORY, BRANCHLINE)
        // Using limit=1000 to get all zones in one request
        const zonesResponse = await fetch(`${backendURL}/static/zones?types=MAINLINE,SIDESTORY,BRANCHLINE&limit=1000`, {
            method: "GET",
            headers: { "Content-Type": "application/json" },
        });

        if (!zonesResponse.ok) {
            console.error("Failed to fetch zones:", zonesResponse.status);
            return { notFound: true };
        }

        const zonesData = (await zonesResponse.json()) as ZonesResponse;

        if (!zonesData.zones || !Array.isArray(zonesData.zones)) {
            console.error("Invalid zones data structure");
            return { notFound: true };
        }

        // Fetch stages (exclude GUIDE type only)
        // Using limit=5000 to get all stages in one request
        const stagesResponse = await fetch(`${backendURL}/static/stages?excludeTypes=GUIDE&limit=5000`, {
            method: "GET",
            headers: { "Content-Type": "application/json" },
        });

        if (!stagesResponse.ok) {
            console.error("Failed to fetch stages:", stagesResponse.status);
            return { notFound: true };
        }

        const stagesData = (await stagesResponse.json()) as StagesResponse;

        if (!stagesData.stages || !Array.isArray(stagesData.stages)) {
            console.error("Invalid stages data structure");
            return { notFound: true };
        }

        // Filter stages to only include those in permanent zones
        const permanentZoneIds = new Set(zonesData.zones.map((z) => z.zoneId));
        const filteredStages = stagesData.stages.filter((s) => permanentZoneIds.has(s.zoneId));

        return {
            props: {
                zones: zonesData.zones,
                stages: filteredStages,
            },
        };
    } catch (error) {
        console.error("Failed to fetch randomizer data:", error);
        return { notFound: true };
    }
};

export default RandomizerPage;
