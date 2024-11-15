import { GeistSans } from "geist/font/sans";
import { type AppType } from "next/app";
import Layout from "~/components/layout";
import { ThemeProvider } from "~/components/theme-provider";

import "~/styles/globals.css";

const MyApp: AppType = ({ Component, pageProps }) => {
    return (
        <div className={GeistSans.className}>
            <ThemeProvider
                attribute="class"
                defaultTheme="dark"
                enableSystem
                disableTransitionOnChange
            >
                <Layout>
                    <Component {...pageProps} />
                </Layout>
            </ThemeProvider>
        </div>
    );
};

export default MyApp;
