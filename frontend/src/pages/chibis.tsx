import { ChibiViewer } from "~/components/chibi-viewer";
import Head from "next/head";
import type { NextPage } from "next";

const ChibisPage: NextPage = () => {
    return (
        <>
            <Head>
                <title>Arknights Chibis</title>
                <meta name="description" content="View Arknights character chibis" />
            </Head>

            <main className="min-h-screen bg-background">
                <div className="container mx-auto py-8">
                    <div className="mb-8">
                        <h1 className="text-4xl font-bold">Arknights Chibis</h1>
                        <p className="mt-2 text-muted-foreground">View chibi sprites for Arknights operators</p>
                    </div>

                    <ChibiViewer />
                </div>
            </main>
        </>
    );
};

export default ChibisPage;
