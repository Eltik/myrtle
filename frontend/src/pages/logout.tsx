import Head from "next/head";
import { useEffect } from "react";
import { useCookies } from "react-cookie";
import { toast } from "~/hooks/use-toast";
import { usePlayer } from "~/store";

export default function Logout() {
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    const [_, setCookies] = useCookies(["login"]);

    useEffect(() => {
        usePlayer.setState({ playerData: {} });
        setCookies("login", null, { path: "/" });
        toast({
            title: "Logged out",
            description: "You have been logged out.",
        });

        window.location.href = "/";
    });
    return (
        <>
            <Head>
                <title>myrtle.moe</title>
                <meta name="description" content="Elevate your Arknights experience to the next level." />
                <link rel="icon" href="/favicon.ico" />
            </Head>
            <div className="container flex max-w-screen-xl auto-rows-auto flex-col gap-4 px-4 py-8 md:grid md:grid-cols-12 md:px-8 xl:px-4">
                <h1>Hi</h1>
            </div>
        </>
    );
}
