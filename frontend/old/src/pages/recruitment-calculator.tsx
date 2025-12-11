"use client";

import React from "react";
import Head from "next/head";
import { Recruitment } from "~/components/recruitment-calculator";

const RecruitmentCalculator = () => {
    return (
        <>
            <Head>
                <title>Arknights Recruitment Calculator</title>
                <meta name="title" content="Arknights Recruitment Calculator" />
                <meta name="description" content="Calculate the probability of getting a specific operator or operators in a recruitment." />
                <link rel="icon" href="/favicon.ico" />
            </Head>
            <Recruitment />
        </>
    );
};

export default RecruitmentCalculator;
