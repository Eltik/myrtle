import { type AppType } from "next/dist/shared/lib/utils";
import { ThemeProvider } from "~/components/theme-provider";
import { Toaster } from "~/components/ui/toaster";

import "~/styles/globals.css";

const MyApp: AppType = ({ Component, pageProps }) => {
    return (
        <>
            <ThemeProvider attribute="class" defaultTheme="system" enableSystem disableTransitionOnChange>
                <Component {...pageProps} />
                <Toaster />
            </ThemeProvider>
        </>
    );
};

export default MyApp;
