import Image from "next/image";
import Link from "next/link";
import Navbar from "~/components/navbar";

export default function Home() {
    return (
        <>
            <main className="bg-main-blue-200">
                <Navbar />
                <section className="h-screen w-full py-12 md:py-24 lg:py-32 xl:py-48">
                    <div className="px-4 md:px-6 pt-24">
                        <div className="grid items-center gap-6">
                            <div className="flex flex-col justify-center space-y-4 text-center">
                                <div className="space-y-2">
                                    <h1 className="text-white font-bold text-transparent sm:text-5xl xl:text-6xl/none">Enhance your Arknights experience</h1>
                                    <p className="mx-auto max-w-[600px] text-main-grey-300 md:text-xl">Use real-time data to view your operator&apos;s stats, plan your base, and more.</p>
                                </div>
                                <div className="mx-auto w-full max-w-sm space-y-2">
                                    <Link href={"#main-content"} className="inline-flex h-10 items-center justify-center rounded-md px-8 text-sm font-medium shadow transition-colors focus-visible:outline-none focus-visible:ring-1 disabled:pointer-events-none disabled:opacity-50 bg-gray-50 text-gray-900 hover:bg-gray-50/90 focus-visible:ring-gray-300">
                                        Get Started
                                    </Link>
                                </div>
                            </div>
                        </div>
                    </div>
                </section>
                <div className="flex flex-col items-center justify-center min-h-screen" id="main-content">
                    <section className="w-full py-12 md:py-24 lg:py-32 bg-gray-800 transform skew-y-3 text-white">
                        <div className="container px-4 md:px-6 transform -skew-y-3">
                            <div className="grid gap-6 lg:grid-cols-2 lg:gap-12">
                                <div className="flex flex-col justify-center space-y-4">
                                    <h1 className="text-3xl font-bold tracking-tighter sm:text-5xl xl:text-6xl">
                                        Advanced Arknights Strategy Planner
                                    </h1>
                                    <p className="max-w-[600px] text-gray-400 md:text-xl">
                                        Plan your strategies with ease and precision using our advanced tools and features.
                                    </p>
                                    <div className="w-full max-w-sm space-y-2">
                                        <Link href={"/planner"} className="inline-flex h-10 items-center justify-center rounded-md px-8 text-sm font-medium shadow transition-colors focus-visible:outline-none focus-visible:ring-1 disabled:pointer-events-none disabled:opacity-50 bg-gray-50 text-gray-900 hover:bg-gray-50/90 focus-visible:ring-gray-300">
                                            Start Planning
                                        </Link>
                                    </div>
                                </div>
                                <Image alt="Strategy Planner" className="mx-auto aspect-video overflow-hidden rounded-xl object-cover sm:w-full" height="550" src="https://generated.vusercontent.net/placeholder.svg" width="550" />
                            </div>
                        </div>
                    </section>
                    <section className="w-full py-12 md:py-24 lg:py-32 bg-main-blue-200 transform skew-y-3 text-white">
                        <div className="container px-4 md:px-6 transform -skew-y-3">
                            <div className="grid gap-6 lg:grid-cols-2 lg:gap-12">
                                <Image alt="Interactive UI" className="mx-auto aspect-video overflow-hidden rounded-xl object-cover sm:w-full" height="550" src="https://generated.vusercontent.net/placeholder.svg" width="550" />
                                <div className="flex flex-col justify-center space-y-4">
                                    <h1 className="text-3xl font-bold tracking-tighter sm:text-5xl xl:text-6xl">
                                        Interactive User Interface
                                    </h1>
                                    <p className="max-w-[600px] text-gray-400 md:text-xl">
                                        Our user interface is designed to be intuitive and interactive, making it easy for you to plan your
                                        strategies.
                                    </p>
                                    <div className="w-full max-w-sm space-y-2">
                                        <Link href={"/profile"} className="inline-flex h-10 items-center justify-center rounded-md px-8 text-sm font-medium shadow transition-colors focus-visible:outline-none focus-visible:ring-1 disabled:pointer-events-none disabled:opacity-50 bg-gray-50 text-gray-900 hover:bg-gray-50/90 focus-visible:ring-gray-300">
                                            Your Profile
                                        </Link>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </section>
                    <section className="w-full py-12 md:py-24 lg:py-32 bg-gray-800 transform skew-y-3 text-white">
                        <div className="container px-4 md:px-6 transform -skew-y-3">
                            <div className="grid gap-6 lg:grid-cols-2 lg:gap-12">
                                <div className="flex flex-col justify-center space-y-4">
                                    <h1 className="text-3xl font-bold tracking-tighter sm:text-5xl xl:text-6xl">Comprehensive Database</h1>
                                    <p className="max-w-[600px] text-gray-400 md:text-xl">
                                        Our comprehensive database provides you with all the information you need to plan your strategies
                                        effectively.
                                    </p>
                                    <div className="w-full max-w-sm space-y-2">
                                        <Link href={"/leaderboard"} className="inline-flex h-10 items-center justify-center rounded-md px-8 text-sm font-medium shadow transition-colors focus-visible:outline-none focus-visible:ring-1 disabled:pointer-events-none disabled:opacity-50 bg-gray-50 text-gray-900 hover:bg-gray-50/90 focus-visible:ring-gray-300">
                                            Leaderboards
                                        </Link>
                                    </div>
                                </div>
                                <Image alt="Database" className="mx-auto aspect-video overflow-hidden rounded-xl object-cover sm:w-full" height="550" src="https://generated.vusercontent.net/placeholder.svg" width="550" />
                            </div>
                        </div>
                    </section>
                    <div className="w-full py-12 md-py-24 lg:py-32 text-center text-white">
                        <h1 className="text-3xl font-bold tracking-tighter sm:text-5xl xl:text-6xl">Get Started Now</h1>
                        <p className="md:text-xl text-gray-400 mt-5">
                            Start planning your strategies now with our advanced tools and features.
                        </p>
                        <div className="mx-auto w-full max-w-sm space-y-2 mt-4">
                            <Link href={"#main-content"} className="inline-flex h-10 items-center justify-center rounded-md px-8 text-sm font-medium shadow transition-colors focus-visible:outline-none focus-visible:ring-1 disabled:pointer-events-none disabled:opacity-50 bg-gray-50 text-gray-900 hover:bg-gray-50/90 focus-visible:ring-gray-300">
                                Enhance Your Experience
                            </Link>
                        </div>
                    </div>
                </div>
            </main>
        </>
    );
}
