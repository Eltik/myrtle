import type { GetServerSideProps, NextPage } from "next";
import { SEO } from "~/components/seo";
import { RecruitmentCalculator } from "~/components/tools/recruitment-calculator";
import { backendFetch } from "~/lib/backend-fetch";
import type { GachaData } from "~/types/api/impl/gacha";
import type { GachaTag, RecruitableOperatorWithTags } from "~/types/frontend/impl/tools/recruitment";
import type { Operator } from "~/types/api";

interface Props {
    tags: GachaTag[];
    recruitableOperators: RecruitableOperatorWithTags[];
    initialSelectedTagNames: string[];
}

const RecruitmentPage: NextPage<Props> = ({ tags, recruitableOperators, initialSelectedTagNames }) => {
    return (
        <>
            <SEO
                description="Calculate Arknights recruitment probabilities and find the best tag combinations for desired operators. Optimize your recruitment permits."
                keywords={["recruitment calculator", "recruitment tags", "tag combinations", "recruitment optimization"]}
                path="/tools/recruitment"
                title="Recruitment Calculator"
            />
            <RecruitmentCalculator initialSelectedTagNames={initialSelectedTagNames} recruitableOperators={recruitableOperators} tags={tags} />
        </>
    );
};

export const getServerSideProps: GetServerSideProps<Props> = async (context) => {
    // Parse ?tags query parameter (comma-separated tag names)
    const tagsQuery = context.query.tags;
    const initialSelectedTagNames: string[] =
        typeof tagsQuery === "string" && tagsQuery.trim()
            ? tagsQuery
                  .split(",")
                  .map((t) => decodeURIComponent(t.trim()))
                  .filter(Boolean)
            : [];

    try {
        // v3: Fetch gacha data and operators in parallel
        // /static/gacha returns the full GachaData structure (no sub-routes)
        // /static/operators returns Record<string, Operator>
        const [gachaResponse, operatorsResponse] = await Promise.all([
            backendFetch("/static/gacha"),
            backendFetch("/static/operators"),
        ]);

        if (!gachaResponse.ok) {
            console.error("Failed to fetch gacha data:", gachaResponse.status);
            return { notFound: true };
        }

        if (!operatorsResponse.ok) {
            console.error("Failed to fetch operators:", operatorsResponse.status);
            return { notFound: true };
        }

        const gachaData = (await gachaResponse.json()) as GachaData;
        const operatorsData = (await operatorsResponse.json()) as Record<string, Operator>;

        // Extract recruitment tags from gacha data
        const tags = gachaData.gachaTags;
        if (!tags || !Array.isArray(tags)) {
            console.error("Invalid gacha data structure: missing gachaTags");
            return { notFound: true };
        }

        // Build recruitable operators from recruitDetail and operators data
        // Parse the recruitDetail HTML to extract recruitable operator names
        const recruitDetail = gachaData.recruitDetail ?? "";
        const recruitableOperators: RecruitableOperatorWithTags[] = [];

        // Build a tag name map for looking up tags
        const tagNameMap: Record<string, GachaTag> = {};
        for (const tag of tags) {
            tagNameMap[tag.tagName] = tag;
        }

        // Extract recruitable operator names from recruitDetail
        // The recruitDetail contains operator names in a formatted string
        // We need to match operators that are in the recruitment pool
        const operatorsList = Object.values(operatorsData);

        // Parse recruitDetail to get recruitable operator names
        // Format typically includes names separated by \n or / with ★ rarity indicators
        const recruitableNames = new Set<string>();
        if (recruitDetail) {
            // Extract names from the recruit detail text
            // Typical format: "★★★★★★\n<@rc.eml>Operator1</>\n<@rc.eml>Operator2</>"
            const nameMatches = recruitDetail.match(/<@rc\.eml>(.*?)<\/>/g);
            if (nameMatches) {
                for (const match of nameMatches) {
                    const name = match.replace(/<@rc\.eml>/, "").replace(/<\/>/, "").trim();
                    if (name) recruitableNames.add(name);
                }
            }

            // Also try plain text format with / separator
            if (recruitableNames.size === 0) {
                const lines = recruitDetail.split("\n");
                for (const line of lines) {
                    const names = line.split("/").map((n) => n.replace(/★+/g, "").replace(/<[^>]*>/g, "").trim()).filter(Boolean);
                    for (const name of names) {
                        if (name.length > 0 && !name.startsWith("-----")) {
                            recruitableNames.add(name);
                        }
                    }
                }
            }
        }

        // Map recruitable names to operator data with tags
        for (const operator of operatorsList) {
            if (!operator.id || !recruitableNames.has(operator.name)) continue;

            // Build tag list for this operator based on their properties
            const operatorTags: string[] = [];

            // Add position tag
            if (operator.position === "MELEE") operatorTags.push("Melee");
            if (operator.position === "RANGED") operatorTags.push("Ranged");

            // Add profession-based tags
            const professionTagMap: Record<string, string> = {
                PIONEER: "Vanguard",
                WARRIOR: "Guard",
                TANK: "Defender",
                SNIPER: "Sniper",
                CASTER: "Caster",
                SUPPORT: "Supporter",
                MEDIC: "Medic",
                SPECIAL: "Specialist",
            };
            const profTag = professionTagMap[operator.profession];
            if (profTag) operatorTags.push(profTag);

            // Add rarity-based qualification tags
            const rarity = typeof operator.rarity === "number" ? operator.rarity : Number.parseInt(String(operator.rarity).replace("TIER_", ""), 10);
            if (rarity === 6) operatorTags.push("Top Operator");
            if (rarity === 5) operatorTags.push("Senior Operator");
            if (rarity === 1) operatorTags.push("Robot");

            // Add tagList from operator data if available
            if ((operator as { tagList?: string[] }).tagList) {
                operatorTags.push(...(operator as { tagList: string[] }).tagList);
            }

            const rarityStr = `TIER_${rarity}`;

            recruitableOperators.push({
                id: operator.id ?? "",
                name: operator.name,
                rarity: rarityStr,
                profession: operator.profession,
                position: operator.position,
                tagList: operatorTags,
            });
        }

        return {
            props: {
                tags,
                recruitableOperators,
                initialSelectedTagNames,
            },
        };
    } catch (error) {
        console.error("Failed to fetch recruitment data:", error);
        return { notFound: true };
    }
};

export default RecruitmentPage;
