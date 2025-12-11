"use client";

import Head from "next/head";
import { Recruitment } from "~/components/recruitment-calculator";

const RecruitmentCalculator = () => {
    return (
        <>
            <Head>
                <title>Arknights Recruitment Calculator</title>
                <meta content="Arknights Recruitment Calculator" name="title" />
                <meta content="Calculate the probability of getting a specific operator or operators in a recruitment." name="description" />
                <link href="/favicon.ico" rel="icon" />
            </Head>
            <Recruitment />
        </>
    );
};

export default RecruitmentCalculator;
