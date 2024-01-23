import Link from "next/link";
import Navbar from "~/components/navbar";
import { Badge } from "~/components/ui/badge";
import { Button } from "~/components/ui/button";

export default function Home() {
    return (
        <>
            <main>
                <Navbar />
                <section className="mx-auto flex max-w-[980px] flex-col items-center gap-2 py-8 md:py-12 md:pb-8 lg:py-24 lg:pb-20">
                    <Link href={"https://github.com/Eltik/myrtle.moe"} target="_blank">
                        <Badge className="cursor-pointer">GitHub - v1.0</Badge>
                    </Link>
                    <h1 className="text-center text-3xl font-bold leading-tight tracking-tighter md:text-6xl lg:leading-[1.1]">Experience Arknights like never before.</h1>
                    <span className="max-w-[750px] text-center text-lg text-muted-foreground sm:text-xl">A fan-made website for Arknights that provides a better experience for the game.</span>
                    <div className="flex w-full items-center justify-center space-x-4 py-4 md:pb-10">
                        <Button>
                            <Link href={"/"}>Get Started</Link>
                        </Button>
                        <Button variant={"outline"}>
                            <Link href={"https://github.com/Eltik/myrtle.moe"} target="_blank" className="flex flex-row items-center justify-center">
                                <svg viewBox="0 0 438.549 438.549" className="mr-2 h-4 w-4">
                                    <path fill="currentColor" d="M409.132 114.573c-19.608-33.596-46.205-60.194-79.798-79.8-33.598-19.607-70.277-29.408-110.063-29.408-39.781 0-76.472 9.804-110.063 29.408-33.596 19.605-60.192 46.204-79.8 79.8C9.803 148.168 0 184.854 0 224.63c0 47.78 13.94 90.745 41.827 128.906 27.884 38.164 63.906 64.572 108.063 79.227 5.14.954 8.945.283 11.419-1.996 2.475-2.282 3.711-5.14 3.711-8.562 0-.571-.049-5.708-.144-15.417a2549.81 2549.81 0 01-.144-25.406l-6.567 1.136c-4.187.767-9.469 1.092-15.846 1-6.374-.089-12.991-.757-19.842-1.999-6.854-1.231-13.229-4.086-19.13-8.559-5.898-4.473-10.085-10.328-12.56-17.556l-2.855-6.57c-1.903-4.374-4.899-9.233-8.992-14.559-4.093-5.331-8.232-8.945-12.419-10.848l-1.999-1.431c-1.332-.951-2.568-2.098-3.711-3.429-1.142-1.331-1.997-2.663-2.568-3.997-.572-1.335-.098-2.43 1.427-3.289 1.525-.859 4.281-1.276 8.28-1.276l5.708.853c3.807.763 8.516 3.042 14.133 6.851 5.614 3.806 10.229 8.754 13.846 14.842 4.38 7.806 9.657 13.754 15.846 17.847 6.184 4.093 12.419 6.136 18.699 6.136 6.28 0 11.704-.476 16.274-1.423 4.565-.952 8.848-2.383 12.847-4.285 1.713-12.758 6.377-22.559 13.988-29.41-10.848-1.14-20.601-2.857-29.264-5.14-8.658-2.286-17.605-5.996-26.835-11.14-9.235-5.137-16.896-11.516-22.985-19.126-6.09-7.614-11.088-17.61-14.987-29.979-3.901-12.374-5.852-26.648-5.852-42.826 0-23.035 7.52-42.637 22.557-58.817-7.044-17.318-6.379-36.732 1.997-58.24 5.52-1.715 13.706-.428 24.554 3.853 10.85 4.283 18.794 7.952 23.84 10.994 5.046 3.041 9.089 5.618 12.135 7.708 17.705-4.947 35.976-7.421 54.818-7.421s37.117 2.474 54.823 7.421l10.849-6.849c7.419-4.57 16.18-8.758 26.262-12.565 10.088-3.805 17.802-4.853 23.134-3.138 8.562 21.509 9.325 40.922 2.279 58.24 15.036 16.18 22.559 35.787 22.559 58.817 0 16.178-1.958 30.497-5.853 42.966-3.9 12.471-8.941 22.457-15.125 29.979-6.191 7.521-13.901 13.85-23.131 18.986-9.232 5.14-18.182 8.85-26.84 11.136-8.662 2.286-18.415 4.004-29.263 5.146 9.894 8.562 14.842 22.077 14.842 40.539v60.237c0 3.422 1.19 6.279 3.572 8.562 2.379 2.279 6.136 2.95 11.276 1.995 44.163-14.653 80.185-41.062 108.068-79.226 27.88-38.161 41.825-81.126 41.825-128.906-.01-39.771-9.818-76.454-29.414-110.049z" />
                                </svg>
                                GitHub
                            </Link>
                        </Button>
                    </div>
                </section>
                <section className="mx-auto mt-24 flex w-full flex-col items-center justify-center bg-zinc-950/30 px-12">
                    <div className="mx-auto max-w-7xl px-4 py-6 sm:px-6">
                        <div className="text-center">
                            <p className="text-base font-bold uppercase leading-6 tracking-wide text-secondary-foreground">Contemporary and Modern</p>
                            <h3 className="text-4xl font-extrabold">Changing the Arknights Community</h3>
                            <p className="mt-3 text-base text-secondary-foreground/75">Open-sourced, trendsetting, and reliable. myrtle.moe aims to create an easily-accessible tool for playing Arknights.</p>
                        </div>
                        <ul className="items-center justify-center md:grid md:grid-cols-2 md:gap-x-8 md:gap-y-10">
                            <li className="md_mt-0 mt-10">
                                <div className="flex">
                                    <div className="shrink-0">
                                        <div className="bg-main flex h-12 w-12 items-center justify-center rounded-md text-white">
                                            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="h-6 w-6 stroke-current">
                                                <path d="M15 14c.2-1 .7-1.7 1.5-2.5 1-.9 1.5-2.2 1.5-3.5A6 6 0 0 0 6 8c0 1 .2 2.2 1.5 3.5.7.7 1.3 1.5 1.5 2.5" />
                                                <path d="M9 18h6" />
                                                <path d="M10 22h4" />
                                            </svg>
                                        </div>
                                    </div>
                                    <div className="ml-4">
                                        <h5 className="font-semibold">Experienced</h5>
                                        <p className="dark_text-gray-400 mt-2 text-base leading-6 text-secondary-foreground/75">Recoded and redesigned with the most popular anime, manga, and light novel websites in mind over many years.</p>
                                    </div>
                                </div>
                            </li>
                            <li className="md_mt-0 mt-10">
                                <div className="flex">
                                    <div className="shrink-0">
                                        <div className="bg-main flex h-12 w-12 items-center justify-center rounded-md text-white">
                                            <svg className="h-6 w-6 stroke-current" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                                                <path d="M3 3v18h18" />
                                                <path d="m19 9-5 5-4-4-3 3" />
                                            </svg>
                                        </div>
                                    </div>
                                    <div className="ml-4">
                                        <h5 className="font-semibold">Continously Improved</h5>
                                        <p className="dark_text-gray-400 mt-2 text-base leading-6 text-secondary-foreground/75">Anify is built on the basis of tailoring each feature to what the community wants. Features are added based on what is popular and what is trending in the anime/manga/novel community.</p>
                                    </div>
                                </div>
                            </li>
                            <li className="md_mt-0 mt-10">
                                <div className="flex">
                                    <div className="shrink-0">
                                        <div className="bg-main flex h-12 w-12 items-center justify-center rounded-md text-white">
                                            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="h-6 w-6 stroke-current">
                                                <path d="M3.85 8.62a4 4 0 0 1 4.78-4.77 4 4 0 0 1 6.74 0 4 4 0 0 1 4.78 4.78 4 4 0 0 1 0 6.74 4 4 0 0 1-4.77 4.78 4 4 0 0 1-6.75 0 4 4 0 0 1-4.78-4.77 4 4 0 0 1 0-6.76Z" />
                                                <path d="m9 12 2 2 4-4" />
                                            </svg>
                                        </div>
                                    </div>
                                    <div className="ml-4">
                                        <h5 className="font-semibold">Reliable</h5>
                                        <p className="dark_text-gray-400 mt-2 text-base leading-6 text-secondary-foreground/75">With the use of multiple providers, if one anime or manga provider is down you can always keep up-to-date with your favorite shows and series.</p>
                                    </div>
                                </div>
                            </li>
                            <li className="md_mt-0 mt-10">
                                <div className="flex">
                                    <div className="shrink-0">
                                        <div className="bg-main flex h-12 w-12 items-center justify-center rounded-md text-white">
                                            <svg className="h-6 w-6 stroke-current" fill="none" viewBox="0 0 24 24">
                                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M13 10V3L4 14h7v7l9-11h-7z"></path>
                                            </svg>
                                        </div>
                                    </div>
                                    <div className="ml-4">
                                        <h5 className="font-semibold">Active Community</h5>
                                        <p className="dark_text-gray-400 mt-2 text-base leading-6 text-secondary-foreground/75">Anify is powered by it&apos;s growing community implementing suggestions as they come so that the website will always be modern and unique compared to other popular websites.</p>
                                    </div>
                                </div>
                            </li>
                        </ul>
                    </div>
                </section>
            </main>
        </>
    );
}
