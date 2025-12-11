import { GeistSans } from "geist/font/sans";
import type { AppType } from "next/app";
import { PagesProgressBar as ProgressBar } from "next-nprogress-bar";
import { CookiesProvider } from "react-cookie";
import Layout from "~/components/layout/layout";
import { ThemeProvider } from "~/components/layout/theme-provider";

import "~/styles/globals.css";

const MyApp: AppType = ({ Component, pageProps }) => {
    return (
        <CookiesProvider>
            <div className={GeistSans.className} suppressHydrationWarning={true}>
                <ThemeProvider attribute="class" defaultTheme="dark" disableTransitionOnChange enableSystem>
                    <Layout>
                        <Component {...pageProps} />
                        <ProgressBar color="#f59a9f" height="4px" options={{ showSpinner: false }} shallowRouting />
                    </Layout>
                </ThemeProvider>
            </div>
        </CookiesProvider>
    );
};

export default MyApp;
