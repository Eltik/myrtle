import { type AppType } from "next/dist/shared/lib/utils";
import { ThemeProvider } from "~/components/theme-provider";
import { Toaster } from "~/components/ui/toaster";

import NProgress from "nprogress";
import "~/styles/nprogress.css";
import "~/styles/globals.css";
import { useRouter } from "next/router";
import { useEffect } from "react";

const MyApp: AppType = ({ Component, pageProps }) => {
    const router = useRouter();
    useEffect(() => {
        const handleRouteChange = () => {
            NProgress.done();
        };
        router.events.on("routeChangeComplete", handleRouteChange);
        router.events.on("routeChangeStart", () => NProgress.start());
        router.events.on("routeChangeError", () => NProgress.done());
        return () => {
            router.events.off("routeChangeComplete", handleRouteChange);
        };
    }, [router.events]);

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
