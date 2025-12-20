import type { AppType } from "next/app";
import { Geist } from "next/font/google";
import { ThemeProvider } from "next-themes";
import { PagesTopLoader } from "nextjs-toploader/pages";
import { Layout } from "~/components/layout/layout";
import { Toaster } from "~/components/ui/shadcn/sonner";

import "~/styles/globals.css";

const geist = Geist({
    subsets: ["latin"],
});

const MyApp: AppType = ({ Component, pageProps }) => {
    return (
        <ThemeProvider attribute="class" defaultTheme="dark" disableTransitionOnChange enableSystem>
            <div className={geist.className}>
                <PagesTopLoader color="#e66e68" height={4} showSpinner={false} />
                <Layout>
                    <Toaster />
                    <Component {...pageProps} />
                </Layout>
            </div>
        </ThemeProvider>
    );
};

export default MyApp;
