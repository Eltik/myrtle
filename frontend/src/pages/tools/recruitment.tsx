import type { GetServerSideProps, NextPage } from "next";
import Head from "next/head";
import { RecruitmentCalculator } from "~/components/tools/recruitment-calculator";
import type { GachaTag, RecruitableOperatorWithTags } from "~/components/tools/recruitment-calculator/impl/types";
import { env } from "~/env";

interface Props {
    tags: GachaTag[];
    recruitableOperators: RecruitableOperatorWithTags[];
}

const RecruitmentPage: NextPage<Props> = ({ tags, recruitableOperators }) => {
    return (
        <>
            <Head>
                <title>Recruitment Calculator | myrtle.moe</title>
                <meta content="Calculate Arknights recruitment probabilities and find the best tag combinations for desired operators." name="description" />
            </Head>
            <RecruitmentCalculator recruitableOperators={recruitableOperators} tags={tags} />
        </>
    );
};

export const getServerSideProps: GetServerSideProps<Props> = async () => {
    const backendURL = env.BACKEND_URL;

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
            },
        };
    } catch (error) {
        console.error("Failed to fetch recruitment data:", error);
        return { notFound: true };
    }
};

export default RecruitmentPage;
