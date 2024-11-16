import { GeistSans } from "geist/font/sans";
import { type AppType } from "next/app";
import Layout from "~/components/base/layout";
import { ThemeProvider } from "~/components/base/theme-provider";

import "~/styles/globals.css";

const MyApp: AppType = ({ Component, pageProps }) => {
    return (
        <div className={GeistSans.className} suppressHydrationWarning={true}>
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
