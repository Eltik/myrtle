import { type AppType } from "next/dist/shared/lib/utils";
import { ThemeProvider } from "~/components/theme-provider";

import "~/styles/globals.css";

const MyApp: AppType = ({ Component, pageProps }) => {
    return (
        <>
            <ThemeProvider attribute="class" defaultTheme="system" enableSystem disableTransitionOnChange>
                <Component {...pageProps} />
            </ThemeProvider>
        </>
    );
};

export default MyApp;
