import type { AppType } from "next/app";
import { Geist } from "next/font/google";
import { Layout } from "~/components/layout/layout";

import "~/styles/globals.css";

const geist = Geist({
    subsets: ["latin"],
});

const MyApp: AppType = ({ Component, pageProps }) => {
    return (
        <div className={geist.className}>
            <Layout>
                <Component {...pageProps} />
            </Layout>
        </div>
    );
};

export default MyApp;
