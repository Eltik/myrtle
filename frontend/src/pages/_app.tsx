import type { AppType } from "next/app";
import { Geist } from "next/font/google";
import { ThemeProvider } from "next-themes";
import { PagesTopLoader } from "nextjs-toploader/pages";
import { Layout } from "~/components/layout/layout";
import { Toaster } from "~/components/ui/shadcn/sonner";
import { AccentColorProvider } from "~/context/accent-color-context";

import "~/styles/globals.css";

const geist = Geist({
    subsets: ["latin"],
});

const MyApp: AppType = ({ Component, pageProps }) => {
    return (
        <ThemeProvider attribute="class" defaultTheme="dark" disableTransitionOnChange enableSystem>
            <AccentColorProvider>
                <div className={geist.className}>
                    <PagesTopLoader color="var(--primary)" height={4} showSpinner={false} />
                    <Layout>
                        <Toaster />
                        <Component {...pageProps} />
                    </Layout>
                </div>
            </AccentColorProvider>
        </ThemeProvider>
    );
};

export default MyApp;
