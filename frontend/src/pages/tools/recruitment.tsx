import type { GetServerSideProps, NextPage } from "next";
import { SEO } from "~/components/seo";
import { RecruitmentCalculator } from "~/components/tools/recruitment-calculator";
import { env } from "~/env";
import type { GachaTag, RecruitableOperatorWithTags } from "~/types/frontend/impl/tools/recruitment";

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
    const backendURL = env.BACKEND_URL;

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
        // Fetch both tags and recruitable operators in parallel
        const [recruitmentResponse, recruitableResponse] = await Promise.all([
            fetch(`${backendURL}/static/gacha/recruitment`, {
                method: "GET",
                headers: { "Content-Type": "application/json" },
            }),
            fetch(`${backendURL}/static/gacha/recruitable`, {
                method: "GET",
                headers: { "Content-Type": "application/json" },
            }),
        ]);

        if (!recruitmentResponse.ok) {
            console.error("Failed to fetch recruitment data:", recruitmentResponse.status);
            return { notFound: true };
        }

        if (!recruitableResponse.ok) {
            console.error("Failed to fetch recruitable operators:", recruitableResponse.status);
            return { notFound: true };
        }

        const recruitmentData = (await recruitmentResponse.json()) as {
            recruitment: {
                tags: GachaTag[];
                tagMap: Record<number, GachaTag>;
                tagNameMap: Record<string, GachaTag>;
                recruitDetail: string;
                recruitPool: unknown;
            };
        };

        const recruitableData = (await recruitableResponse.json()) as {
            operators: RecruitableOperatorWithTags[];
            total: number;
        };

        // Validate recruitment data
        if (!recruitmentData.recruitment?.tags || !Array.isArray(recruitmentData.recruitment.tags)) {
            console.error("Invalid recruitment data structure");
            return { notFound: true };
        }

        // Validate recruitable operators data
        if (!recruitableData.operators || !Array.isArray(recruitableData.operators)) {
            console.error("Invalid recruitable operators data structure");
            return { notFound: true };
        }

        return {
            props: {
                tags: recruitmentData.recruitment.tags,
                recruitableOperators: recruitableData.operators,
                initialSelectedTagNames,
            },
        };
    } catch (error) {
        console.error("Failed to fetch recruitment data:", error);
        return { notFound: true };
    }
};

export default RecruitmentPage;
