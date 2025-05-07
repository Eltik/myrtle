import { GeistSans } from "geist/font/sans";
import { type AppType } from "next/app";
import Layout from "~/components/layout/layout";
import { ThemeProvider } from "~/components/layout/theme-provider";
import { CookiesProvider } from "react-cookie";
import { PagesProgressBar as ProgressBar } from "next-nprogress-bar";

import "~/styles/globals.css";

const MyApp: AppType = ({ Component, pageProps }) => {
    return (
        <CookiesProvider>
            <div className={GeistSans.className} suppressHydrationWarning={true}>
                <ThemeProvider attribute="class" defaultTheme="dark" enableSystem disableTransitionOnChange>
                    <Layout>
                        <Component {...pageProps} />
                        <ProgressBar height="4px" color="#f59a9f" options={{ showSpinner: false }} shallowRouting />
                    </Layout>
                </ThemeProvider>
            </div>
        </CookiesProvider>
    );
};

export default MyApp;
