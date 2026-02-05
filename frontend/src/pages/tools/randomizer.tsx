import type { GetServerSideProps, NextPage } from "next";
import { SEO } from "~/components/seo";
import type { RandomizerOperator } from "~/components/tools/randomizer";
import { Randomizer } from "~/components/tools/randomizer";
import { env } from "~/env";
import type { Stage, StagesResponse } from "~/types/api/impl/stage";
import type { Zone, ZonesResponse } from "~/types/api/impl/zone";

interface OperatorsResponse {
    operators: RandomizerOperator[];
    has_more: boolean;
    next_cursor: string | null;
    total: number;
}

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
    const backendURL = env.BACKEND_URL;

    try {
        const operatorFields = ["id", "name", "rarity", "profession", "subProfessionId", "position", "portrait"].join(",");
        const operatorParams = new URLSearchParams({
            limit: "1000",
            fields: operatorFields,
        });

        const [zonesResponse, stagesResponse, operatorsResponse] = await Promise.all([
            fetch(`${backendURL}/static/zones?types=MAINLINE,ACTIVITY&limit=1000`, {
                method: "GET",
                headers: { "Content-Type": "application/json" },
            }),
            fetch(`${backendURL}/static/stages?excludeTypes=GUIDE&limit=5000`, {
                method: "GET",
                headers: { "Content-Type": "application/json" },
            }),
            fetch(`${backendURL}/static/operators?${operatorParams.toString()}`, {
                method: "GET",
                headers: { "Content-Type": "application/json" },
            }),
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

        const [zonesData, stagesData, operatorsData] = await Promise.all([zonesResponse.json() as Promise<ZonesResponse>, stagesResponse.json() as Promise<StagesResponse>, operatorsResponse.json() as Promise<OperatorsResponse>]);

        if (!zonesData.zones || !Array.isArray(zonesData.zones)) {
            console.error("Invalid zones data structure");
            return { notFound: true };
        }

        if (!stagesData.stages || !Array.isArray(stagesData.stages)) {
            console.error("Invalid stages data structure");
            return { notFound: true };
        }

        if (!operatorsData.operators || !Array.isArray(operatorsData.operators)) {
            console.error("Invalid operators data structure");
            return { notFound: true };
        }

        const permanentZoneIds = new Set(zonesData.zones.map((z) => z.zoneId));
        const filteredStages = stagesData.stages.filter((s) => permanentZoneIds.has(s.zoneId));

        const playableOperators = operatorsData.operators.filter((op) => op.profession !== "TOKEN" && op.profession !== "TRAP");

        return {
            props: {
                zones: zonesData.zones,
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
