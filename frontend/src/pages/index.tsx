import Image from "next/image";
import Link from "next/link";
import Navbar from "~/components/navbar";

export default function Home() {
    return (
        <>
            <Navbar>
                <main className="bg-main-blue-200">
                    <section className="h-screen w-full py-12 md:py-24 lg:py-32 xl:py-48">
                        <div className="px-4 md:px-6">
                            <div className="grid items-center gap-6">
                                <div className="flex flex-col justify-center space-y-4 text-center">
                                    <div className="space-y-2">
                                        <h1 className="text-white font-bold text-transparent sm:text-5xl xl:text-6xl/none">Enhance your Arknights experience</h1>
                                        <p className="mx-auto max-w-[600px] text-main-grey-300 md:text-xl">Use real-time data to view your operator&apos;s stats, plan your base, and more.</p>
                                    </div>
                                    <div className="mx-auto w-full max-w-sm space-y-2">
                                        <Link href={"/login"} className="inline-flex h-10 items-center justify-center rounded-md px-8 text-sm font-medium shadow transition-colors focus-visible:outline-none focus-visible:ring-1 disabled:pointer-events-none disabled:opacity-50 bg-gray-50 text-gray-900 hover:bg-gray-50/90 focus-visible:ring-gray-300">
                                            Get Started
                                        </Link>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </section>
                    <div className="flex flex-col items-center justify-center min-h-screen">
                        <section className="w-full py-12 md:py-24 lg:py-32 bg-gray-100 dark:bg-gray-800 transform skew-y-3">
                            <div className="container px-4 md:px-6 transform -skew-y-3">
                                <div className="grid gap-6 lg:grid-cols-2 lg:gap-12">
                                    <div className="flex flex-col justify-center space-y-4">
                                        <h1 className="text-3xl font-bold tracking-tighter sm:text-5xl xl:text-6xl">
                                            Advanced Arknights Strategy Planner
                                        </h1>
                                        <p className="max-w-[600px] text-gray-500 md:text-xl dark:text-gray-400">
                                            Plan your strategies with ease and precision using our advanced tools and features.
                                        </p>
                                    </div>
                                    <Image alt="Strategy Planner" className="mx-auto aspect-video overflow-hidden rounded-xl object-cover sm:w-full" height="550" src="https://generated.vusercontent.net/placeholder.svg" width="550" />
                                </div>
                            </div>
                        </section>
                        <section className="w-full py-12 md:py-24 lg:py-32 bg-gray-200 transform skew-y-3">
                            <div className="container px-4 md:px-6 transform -skew-y-3">
                                <div className="grid gap-6 lg:grid-cols-2 lg:gap-12">
                                    <Image alt="Interactive UI" className="mx-auto aspect-video overflow-hidden rounded-xl object-cover sm:w-full" height="550" src="https://generated.vusercontent.net/placeholder.svg" width="550" />
                                    <div className="flex flex-col justify-center space-y-4">
                                    <h1 className="text-3xl font-bold tracking-tighter sm:text-5xl xl:text-6xl">
                                        Interactive User Interface
                                    </h1>
                                    <p className="max-w-[600px] text-gray-500 md:text-xl dark:text-gray-400">
                                        Our user interface is designed to be intuitive and interactive, making it easy for you to plan your
                                        strategies.
                                    </p>
                                    </div>
                                </div>
                            </div>
                        </section>
                        <section className="w-full py-12 md:py-24 lg:py-32 bg-gray-100 dark:bg-gray-800 transform skew-y-3">
                            <div className="container px-4 md:px-6 transform -skew-y-3">
                                <div className="grid gap-6 lg:grid-cols-2 lg:gap-12">
                                    <div className="flex flex-col justify-center space-y-4">
                                    <h1 className="text-3xl font-bold tracking-tighter sm:text-5xl xl:text-6xl">Comprehensive Database</h1>
                                    <p className="max-w-[600px] text-gray-500 md:text-xl dark:text-gray-400">
                                        Our comprehensive database provides you with all the information you need to plan your strategies
                                        effectively.
                                    </p>
                                    </div>
                                    <Image alt="Database" className="mx-auto aspect-video overflow-hidden rounded-xl object-cover sm:w-full" height="550" src="https://generated.vusercontent.net/placeholder.svg" width="550" />
                                </div>
                            </div>
                        </section>
                    </div>
                </main>
            </Navbar>
        </>
    );
}
