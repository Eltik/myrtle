import type { AppType } from "next/app";
import { Geist } from "next/font/google";
import { PagesTopLoader } from "nextjs-toploader/pages";
import { Layout } from "~/components/layout/layout";

import "~/styles/globals.css";

const geist = Geist({
    subsets: ["latin"],
});

const MyApp: AppType = ({ Component, pageProps }) => {
    return (
        <div className={geist.className}>
            <PagesTopLoader color="#e66e68" height={4} showSpinner={false} />
            <Layout>
                <Component {...pageProps} />
            </Layout>
        </div>
    );
};

export default MyApp;
