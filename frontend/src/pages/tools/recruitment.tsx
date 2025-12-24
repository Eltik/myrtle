import type { GetServerSideProps, NextPage } from "next";
import Head from "next/head";
import { RecruitmentCalculator } from "~/components/tools/recruitment-calculator";
import type { GachaTag } from "~/components/tools/recruitment-calculator/impl/types";
import { env } from "~/env";

interface Props {
    tags: GachaTag[];
}

const RecruitmentPage: NextPage<Props> = ({ tags }) => {
    return (
        <>
            <Head>
                <title>Recruitment Calculator | myrtle.moe</title>
                <meta content="Calculate Arknights recruitment probabilities and find the best tag combinations for desired operators." name="description" />
            </Head>
            <RecruitmentCalculator tags={tags} />
        </>
    );
};

export const getServerSideProps: GetServerSideProps<Props> = async () => {
    const backendURL = env.BACKEND_URL;

    try {
        // Fetch recruitment tags
        const recruitmentResponse = await fetch(`${backendURL}/static/gacha/recruitment`, {
            method: "GET",
            headers: { "Content-Type": "application/json" },
        });

        if (!recruitmentResponse.ok) {
            console.error("Failed to fetch recruitment data:", recruitmentResponse.status);
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

        // Validate recruitment data
        if (!recruitmentData.recruitment?.tags || !Array.isArray(recruitmentData.recruitment.tags)) {
            console.error("Invalid recruitment data structure");
            return { notFound: true };
        }

        return {
            props: {
                tags: recruitmentData.recruitment.tags,
            },
        };
    } catch (error) {
        console.error("Failed to fetch recruitment data:", error);
        return { notFound: true };
    }
};

export default RecruitmentPage;
