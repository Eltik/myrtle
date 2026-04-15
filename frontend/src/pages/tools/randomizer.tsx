import type { GetServerSideProps, NextPage } from "next";
import { SEO } from "~/components/seo";
import type { RandomizerOperator } from "~/components/tools/randomizer";
import { Randomizer } from "~/components/tools/randomizer";
import { backendFetch } from "~/lib/backend-fetch";
import type { Operator } from "~/types/api";
import type { Stage } from "~/types/api/impl/stage";
import type { Zone, ZoneType } from "~/types/api/impl/zone";

interface Props {
    zones: Zone[];
    stages: Stage[];
    operators: RandomizerOperator[];
}

const RandomizerPage: NextPage<Props> = ({ zones, stages, operators }) => {
    return (
        <>
            <SEO
                description="Randomize Arknights stages and operators for challenge runs. Create random squad compositions and test your skills with surprise operator selections."
                keywords={["randomizer", "random squad", "challenge run", "random operators", "random stage"]}
                path="/tools/randomizer"
                title="Randomizer"
            />
            <Randomizer operators={operators} stages={stages} zones={zones} />
        </>
    );
};

export const getServerSideProps: GetServerSideProps<Props> = async () => {
    try {
        // v3: All static endpoints return raw data structures (no query params)
        const [zonesResponse, stagesResponse, operatorsResponse] = await Promise.all([
            backendFetch("/static/zones"),
            backendFetch("/static/stages"),
            backendFetch("/static/operators"),
        ]);

        if (!zonesResponse.ok) {
            console.error("Failed to fetch zones:", zonesResponse.status);
            return { notFound: true };
        }

        if (!stagesResponse.ok) {
            console.error("Failed to fetch stages:", stagesResponse.status);
            return { notFound: true };
        }

        if (!operatorsResponse.ok) {
            console.error("Failed to fetch operators:", operatorsResponse.status);
            return { notFound: true };
        }

        // v3: Returns Record<string, Zone>, Record<string, Stage>, Record<string, Operator>
        const [zonesData, stagesData, operatorsData] = await Promise.all([
            zonesResponse.json() as Promise<Record<string, Zone>>,
            stagesResponse.json() as Promise<Record<string, Stage>>,
            operatorsResponse.json() as Promise<Record<string, Operator>>,
        ]);

        const allZones = Object.values(zonesData);
        const allStages = Object.values(stagesData);
        const allOperators = Object.values(operatorsData);

        // Filter zones client-side (previously done via ?types=MAINLINE,ACTIVITY)
        const allowedZoneTypes: Set<ZoneType> = new Set(["MAINLINE", "ACTIVITY"]);
        const filteredZones = allZones.filter((z) => allowedZoneTypes.has(z.type));

        // Filter stages client-side (previously done via ?excludeTypes=GUIDE)
        const filteredStagesAll = allStages.filter((s) => s.stageType !== "GUIDE");

        // Further filter stages to only those in the allowed zones
        const permanentZoneIds = new Set(filteredZones.map((z) => z.zoneId));
        const filteredStages = filteredStagesAll.filter((s) => permanentZoneIds.has(s.zoneId));

        // Filter out non-playable operators (TOKEN, TRAP)
        const playableOperators = allOperators.filter((op) => op.profession !== "TOKEN" && op.profession !== "TRAP") as RandomizerOperator[];

        return {
            props: {
                zones: filteredZones,
                stages: filteredStages,
                operators: playableOperators,
            },
        };
    } catch (error) {
        console.error("Failed to fetch randomizer data:", error);
        return { notFound: true };
    }
};

export default RandomizerPage;
