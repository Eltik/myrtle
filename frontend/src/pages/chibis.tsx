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
                        <p className="mt-2 text-muted-foreground">View chibi sprites for Arknights operators.</p>
                        <p className="mt-2 text-sm text-muted-foreground md:max-w-[40%]">
                            <b>Note:</b> Arknights has changed the sprite loading for newer operators. If you think that the skin or chibi for an operator looks off, that&apos;s probably why. CN operators are also included here, so the name for the operator may be off.
                        </p>
                    </div>

                    <ChibiViewer />
                </div>
            </main>
        </>
    );
};

export default ChibisPage;
