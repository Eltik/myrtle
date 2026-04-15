import type { GetServerSideProps, NextPage } from "next";
import { SEO } from "~/components/seo";
import { DpsCalculator } from "~/components/tools/dps-calculator";
import { backendFetch } from "~/lib/backend-fetch";
import type { DpsConditionalInfo, DpsOperatorListEntry } from "~/types/api/impl/dps-calculator";
import type { Operator } from "~/types/api/impl/operator";

// v3 raw DPS operator shape (minimal, camelCase from Rust backend)
interface V3DpsOperator {
    id: string;
    name: string;
    availableSkills: number[];
    availableModules: number[];
    defaultSkill: number;
    defaultModule: number;
    conditionals: Array<{
        conditionalType: string;
        name: string;
        default: boolean;
        skills: number[];
        modules: number[];
    }>;
}

const RARITY_TO_NUMBER: Record<string, number> = {
    TIER_1: 1, TIER_2: 2, TIER_3: 3, TIER_4: 4, TIER_5: 5, TIER_6: 6,
};

const MAX_PROMOTION_BY_RARITY: Record<number, number> = {
    1: 0, 2: 0, 3: 1, 4: 2, 5: 2, 6: 2,
};

interface Props {
    operators: DpsOperatorListEntry[];
}

const DpsCalculatorPage: NextPage<Props> = ({ operators }) => {
    return (
        <>
            <SEO
                description="Calculate and compare operator DPS in Arknights with 100% accuracy. Generate DPS curves against different enemy defense and resistance values. Supports 280+ operators."
                keywords={["DPS calculator", "damage calculator", "operator damage", "DPS comparison", "enemy defense"]}
                path="/tools/dps"
                title="DPS Calculator"
            />
            <DpsCalculator operators={operators} />
        </>
    );
};

export const getServerSideProps: GetServerSideProps<Props> = async () => {
    try {
        // v3: Fetch DPS list and static operator data in parallel for enrichment
        const [dpsResponse, operatorsResponse] = await Promise.all([
            backendFetch("/dps/operators"),
            backendFetch("/static/operators"),
        ]);

        if (!dpsResponse.ok || !operatorsResponse.ok) {
            console.error("Failed to fetch DPS data:", dpsResponse.status, operatorsResponse.status);
            return { notFound: true };
        }

        const v3Operators = (await dpsResponse.json()) as V3DpsOperator[];
        const staticOperators = (await operatorsResponse.json()) as Record<string, Operator>;

        if (!Array.isArray(v3Operators)) {
            return { notFound: true };
        }

        // Enrich v3 minimal data with static operator info (rarity, profession, etc.)
        const operators: DpsOperatorListEntry[] = v3Operators.map((op) => {
            const staticOp = staticOperators[op.id];
            const rarity = staticOp?.rarity ? (RARITY_TO_NUMBER[staticOp.rarity] ?? 1) : 1;
            const maxPromotion = MAX_PROMOTION_BY_RARITY[rarity] ?? 2;

            // Map v3 conditional fields (skills/modules) to frontend expected names
            const conditionals: DpsConditionalInfo[] = (op.conditionals ?? []).map((c) => ({
                conditionalType: c.conditionalType as DpsConditionalInfo["conditionalType"],
                name: c.name,
                default: c.default,
                inverted: false,
                applicableSkills: c.skills ?? [],
                applicableModules: c.modules ?? [],
                minElite: 0,
                minModuleLevel: 0,
            }));

            return {
                id: op.id,
                name: op.name,
                calculatorName: op.id,
                rarity,
                profession: staticOp?.profession ?? "",
                availableSkills: op.availableSkills ?? [],
                availableModules: op.availableModules ?? [],
                defaultSkillIndex: op.defaultSkill ?? 1,
                defaultModuleIndex: op.defaultModule ?? 0,
                defaultPotential: 1,
                maxPromotion,
                conditionals,
            };
        });

        return {
            props: {
                operators,
            },
        };
    } catch (error) {
        console.error("Failed to fetch DPS calculator data:", error);
        return { notFound: true };
    }
};

export default DpsCalculatorPage;
