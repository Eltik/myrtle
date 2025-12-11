import type { NextPage } from "next";
import Head from "next/head";
import { ChibiViewer } from "~/components/chibi-viewer";

const ChibisPage: NextPage = () => {
    return (
        <>
            <Head>
                <title>Arknights Chibis</title>
                <meta content="View Arknights character chibis" name="description" />
            </Head>

            <main className="min-h-screen bg-background">
                <div className="container mx-auto py-8">
                    <div className="mb-8">
                        <h1 className="font-bold text-4xl">Arknights Chibis</h1>
                        <p className="mt-2 text-muted-foreground">View chibi sprites for Arknights operators.</p>
                    </div>

                    <ChibiViewer />
                </div>
            </main>
        </>
    );
};

export default ChibisPage;
