import Link from "next/link";
import Navbar from "~/components/navbar";

export default function Home() {
    return (
        <>
            <Navbar>
                <main className="bg-main-blue-100">
                    <section className="w-full h-screen py-12 md:py-24 lg:py-32 xl:py-48">
                        <div className="px-4 md:px-6">
                            <div className="grid gap-6 items-center">
                                <div className="flex flex-col justify-center space-y-4 text-center">
                                    <div className="space-y-2">
                                        <h1 className="text-3xl font-bold tracking-tighter sm:text-5xl xl:text-6xl/none bg-clip-text text-transparent bg-gradient-to-r from-white to-gray-500">
                                            Enhance your Arknights experience
                                        </h1>
                                        <p className="max-w-[600px] text-zinc-200 md:text-xl dark:text-zinc-100 mx-auto">
                                            Use real-time data to view your operator&apos;s stats, plan your base, and more.
                                        </p>
                                    </div>
                                    <div className="w-full max-w-sm space-y-2 mx-auto">
                                        <Link href={"/login"} className="inline-flex h-10 items-center justify-center rounded-md bg-gray-900 px-8 text-sm font-medium text-gray-50 shadow transition-colors hover:bg-gray-900/90 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-gray-950 disabled:pointer-events-none disabled:opacity-50 dark:bg-gray-50 dark:text-gray-900 dark:hover:bg-gray-50/90 dark:focus-visible:ring-gray-300">Get Started</Link>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </section>
                </main>
            </Navbar>
        </>
    );
}
