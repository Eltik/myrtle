import type { NextPage } from "next";
import Head from "next/head";
import { RecruitmentCalculator } from "~/components/tools/recruitment-calculator";

const RecruitmentPage: NextPage = () => {
    return (
        <>
            <Head>
                <title>Recruitment Calculator | myrtle.moe</title>
                <meta content="Calculate Arknights recruitment probabilities and find the best tag combinations for desired operators." name="description" />
            </Head>
            <RecruitmentCalculator />
        </>
    );
};

export default RecruitmentPage;
