import { LogIn, Users } from "lucide-react";
import Head from "next/head";
import { HomeChart } from "~/components/home/home-chart";
import { Button } from "~/components/ui/button";
import { motion } from "framer-motion";

export default function Home() {
    const fadeIn = {
        hidden: { opacity: 0, y: -20 },
        visible: { opacity: 1, y: 0 },
    };

    return (
        <>
            <Head>
                <title>myrtle.moe</title>
                <meta name="title" content="myrtle.moe" />
                <meta name="description" content="Elevate your Arknights experience to the next level." />
                <link rel="icon" href="/favicon.ico" />
            </Head>
            <motion.div
                initial="hidden"
                animate="visible"
                exit={"hidden"}
                variants={{
                    hidden: { opacity: 0 },
                    visible: {
                        opacity: 1,
                        transition: { staggerChildren: 0.2 },
                    },
                }}
                className="container flex max-w-screen-xl auto-rows-auto flex-col gap-4 px-4 py-8 md:grid md:grid-cols-12 md:px-8 xl:px-4"
            >
                <motion.div variants={fadeIn} className="col-span-full rounded-xl border bg-card text-card-foreground shadow">
                    <div className="flex flex-col space-y-1.5 p-6 pb-2">
                        <h2 className="scroll-m-20 pb-0 text-3xl font-bold tracking-tight lg:text-xl xl:text-4xl">Elevate your Arknights experience to the next level.</h2>
                    </div>
                    <p className="p-6 pt-0">An advanced Arknights toolkit for the modern player. Manage your operators, track your progress, and more.</p>
                </motion.div>
                <motion.div variants={fadeIn} className="relative col-span-full h-full rounded-xl border bg-card text-card-foreground shadow xl:col-span-8">
                    <div className="flex scroll-m-20 flex-col p-6">
                        <span className="text-xl font-bold tracking-tight">Get Started</span>
                        <p className="mt-2">Create an account to get started with your Arknights journey.</p>
                    </div>
                    <div className="flex flex-col">
                        <ol className="relative ms-3 mt-4">
                            <li className="mb-10 ms-8 flex items-center">
                                <span className="absolute -start-0 flex h-8 w-8 items-center justify-center rounded-full bg-accent p-3">
                                    <p className="font-bold text-white">1</p>
                                </span>
                                <div className="ms-4 flex flex-col">
                                    <span className="text-lg font-semibold">Login</span>
                                    <p>Login with a YoStar-bounded Arknights account to get started.</p>
                                    <Button className="flex w-28 flex-row justify-center">
                                        <LogIn />
                                        Login
                                    </Button>
                                </div>
                            </li>
                            <li className="mb-10 ms-8 flex items-center">
                                <span className="absolute -start-0 flex h-8 w-8 items-center justify-center rounded-full bg-accent p-3">
                                    <p className="font-bold text-white">2</p>
                                </span>
                                <div className="ms-4 flex flex-col">
                                    <span className="text-lg font-bold">View Your Profile</span>
                                    <p>Check out your profile built with myrtle.moe using your Arknights data.</p>
                                    <Button className="flex w-28 flex-row justify-center">
                                        <Users />
                                        Profile
                                    </Button>
                                </div>
                            </li>
                            <li className="mb-10 ms-8 flex items-center">
                                <span className="absolute -start-0 flex h-8 w-8 items-center justify-center rounded-full bg-accent p-3">
                                    <p className="font-bold text-white">3</p>
                                </span>
                                <div className="ms-4 flex flex-col">
                                    <span className="text-lg font-bold">Give us a star!</span>
                                    <p>To support the development of myrtle.moe, consider giving us a star on GitHub.</p>
                                    <Button className="flex w-28 flex-row justify-center">
                                        <svg viewBox="0 0 438.549 438.549" className="h-4 w-4">
                                            <path fill="currentColor" d="M409.132 114.573c-19.608-33.596-46.205-60.194-79.798-79.8-33.598-19.607-70.277-29.408-110.063-29.408-39.781 0-76.472 9.804-110.063 29.408-33.596 19.605-60.192 46.204-79.8 79.8C9.803 148.168 0 184.854 0 224.63c0 47.78 13.94 90.745 41.827 128.906 27.884 38.164 63.906 64.572 108.063 79.227 5.14.954 8.945.283 11.419-1.996 2.475-2.282 3.711-5.14 3.711-8.562 0-.571-.049-5.708-.144-15.417a2549.81 2549.81 0 01-.144-25.406l-6.567 1.136c-4.187.767-9.469 1.092-15.846 1-6.374-.089-12.991-.757-19.842-1.999-6.854-1.231-13.229-4.086-19.13-8.559-5.898-4.473-10.085-10.328-12.56-17.556l-2.855-6.57c-1.903-4.374-4.899-9.233-8.992-14.559-4.093-5.331-8.232-8.945-12.419-10.848l-1.999-1.431c-1.332-.951-2.568-2.098-3.711-3.429-1.142-1.331-1.997-2.663-2.568-3.997-.572-1.335-.098-2.43 1.427-3.289 1.525-.859 4.281-1.276 8.28-1.276l5.708.853c3.807.763 8.516 3.042 14.133 6.851 5.614 3.806 10.229 8.754 13.846 14.842 4.38 7.806 9.657 13.754 15.846 17.847 6.184 4.093 12.419 6.136 18.699 6.136 6.28 0 11.704-.476 16.274-1.423 4.565-.952 8.848-2.383 12.847-4.285 1.713-12.758 6.377-22.559 13.988-29.41-10.848-1.14-20.601-2.857-29.264-5.14-8.658-2.286-17.605-5.996-26.835-11.14-9.235-5.137-16.896-11.516-22.985-19.126-6.09-7.614-11.088-17.61-14.987-29.979-3.901-12.374-5.852-26.648-5.852-42.826 0-23.035 7.52-42.637 22.557-58.817-7.044-17.318-6.379-36.732 1.997-58.24 5.52-1.715 13.706-.428 24.554 3.853 10.85 4.283 18.794 7.952 23.84 10.994 5.046 3.041 9.089 5.618 12.135 7.708 17.705-4.947 35.976-7.421 54.818-7.421s37.117 2.474 54.823 7.421l10.849-6.849c7.419-4.57 16.18-8.758 26.262-12.565 10.088-3.805 17.802-4.853 23.134-3.138 8.562 21.509 9.325 40.922 2.279 58.24 15.036 16.18 22.559 35.787 22.559 58.817 0 16.178-1.958 30.497-5.853 42.966-3.9 12.471-8.941 22.457-15.125 29.979-6.191 7.521-13.901 13.85-23.131 18.986-9.232 5.14-18.182 8.85-26.84 11.136-8.662 2.286-18.415 4.004-29.263 5.146 9.894 8.562 14.842 22.077 14.842 40.539v60.237c0 3.422 1.19 6.279 3.572 8.562 2.379 2.279 6.136 2.95 11.276 1.995 44.163-14.653 80.185-41.062 108.068-79.226 27.88-38.161 41.825-81.126 41.825-128.906-.01-39.771-9.818-76.454-29.414-110.049z" />
                                        </svg>
                                        GitHub
                                    </Button>
                                </div>
                            </li>
                        </ol>
                    </div>
                </motion.div>
                <section className="col-span-full flex flex-col gap-4 xl:col-span-4">
                    <motion.div variants={fadeIn} className="relative flex-1 rounded-xl border bg-card text-card-foreground shadow">
                        <div className="flex flex-col gap-0 space-y-0 p-6 pb-2">
                            <h2 className="scroll-m-20 pb-0 text-lg font-bold tracking-tight lg:text-xl xl:text-2xl">Statistics & Data</h2>
                            <p className="text-sm leading-7 sm:text-base">View our overall website statistics and data.</p>
                        </div>
                        <HomeChart />
                    </motion.div>
                    <motion.div variants={fadeIn} className="relative flex-1 rounded-xl border bg-card text-card-foreground shadow">
                        <div className="flex flex-col gap-0 space-y-0 p-6 pb-2">
                            <h2 className="scroll-m-20 pb-0 text-lg font-bold tracking-tight lg:text-xl xl:text-2xl">Join Our Community!</h2>
                            <p className="text-sm leading-7 sm:text-base">Curious about how myrtle.moe got started? Have questions about Arknights? Join our welcoming and helpful community below!</p>
                        </div>
                        <div className="flex flex-row flex-wrap gap-4 p-6">
                            <Button>Discord</Button>
                            <Button>Reddit</Button>
                            <Button>Twitter</Button>
                        </div>
                    </motion.div>
                </section>
            </motion.div>
        </>
    );
}
