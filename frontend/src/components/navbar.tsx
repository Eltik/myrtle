import Link from "next/link";

function Navbar({ active }: { active: "home" | "players" | "statistics" | "login" }) {
    return (
        <>
            <header
                className="sticky top-0 flex h-16 w-full flex-col justify-around bg-main-blue-200/80 p-[0_24px] backdrop-blur-sm"
                style={{
                    transform: "translateZ(0)",
                    boxShadow: "inset 0 -1px 0 0 #333",
                }}
            >
                <nav
                    className="relative flex w-full items-center"
                    style={{
                        flex: "1 1",
                    }}
                >
                    <div className="flex w-full items-center justify-between md:hidden ">
                        <div className="flex flex-row items-center justify-start gap-1">
                            <Link href={"/"}>
                                <svg aria-label="Vercel logomark" height="22" role="img" className="w-auto overflow-visible" viewBox="0 0 74 64">
                                    <path d="M37.5896 0.25L74.5396 64.25H0.639648L37.5896 0.25Z" fill="#fff"></path>
                                </svg>
                            </Link>
                            <Link href={"/"} target="_blank" className="flex text-white">
                                myrtle.moe
                            </Link>
                        </div>
                        <div className="flex items-center">
                            <button type="button" className="background-none flex justify-center p-0 text-gray-200">
                                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="">
                                    <circle cx="11" cy="11" r="8" />
                                    <path d="m21 21-4.3-4.3" />
                                </svg>
                            </button>
                            <button type="button" className="m-[0_-5px_0_5px] flex cursor-pointer justify-center rounded-md border-none bg-transparent text-gray-200">
                                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="">
                                    <line x1="4" x2="20" y1="12" y2="12" />
                                    <line x1="4" x2="20" y1="6" y2="6" />
                                    <line x1="4" x2="20" y1="18" y2="18" />
                                </svg>
                            </button>
                        </div>
                    </div>
                    <div className="hidden w-full items-center gap-6 md:flex">
                        <div className="flex flex-row items-center justify-start gap-4">
                            <Link href={"/"}>
                                <svg aria-label="Vercel logomark" height="22" role="img" className="w-auto overflow-visible" viewBox="0 0 74 64">
                                    <path d="M37.5896 0.25L74.5396 64.25H0.639648L37.5896 0.25Z" fill="#fff"></path>
                                </svg>
                            </Link>
                            <Link href={"/"} target="_blank" className="flex text-white">
                                myrtle.moe
                            </Link>
                        </div>
                        <Link href={"/players"} className="relative ml-5 text-main-pink-400 transition-all duration-150 ease-in-out hover:text-main-dark-pink-400">
                            Players
                        </Link>
                        <Link href={"/statistics"} className="relative ml-5 text-main-pink-400 transition-all duration-150 ease-in-out hover:text-main-dark-pink-400">
                            Statistics
                        </Link>
                    </div>
                    <div className="hidden items-center gap-2 md:flex">
                        <button type="button" className="flex h-8 cursor-pointer items-center justify-between whitespace-nowrap rounded-md bg-main-blue-500 p-[0_6px_0_8px] text-gray-50 transition-all duration-150 ease-in-out hover:bg-main-blue-500/70">
                            Search players...
                            <kbd
                                className="ml-4 h-5 rounded-md bg-main-blue-200 p-[0_6px] text-xs text-gray-50"
                                style={{
                                    lineHeight: "20px",
                                    boxShadow: "0 0 0 1px hsla(0,0%,100%,.145)",
                                }}
                            >
                                ⌘K
                            </kbd>
                        </button>
                        <Link href={"/login"} className="relative m-0 flex h-8 max-w-full cursor-pointer items-center justify-center rounded-md border-0 bg-main-blue-400 p-[0_12px] font-bold text-gray-50 outline-none transition-all duration-150 ease-in-out hover:bg-main-blue-400/70">
                            <span className="mr-1 flex min-w-5 items-center justify-center">
                                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" stroke-linejoin="round" className="">
                                    <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4" />
                                    <polyline points="10 17 15 12 10 7" />
                                    <line x1="15" x2="3" y1="12" y2="12" />
                                </svg>
                            </span>
                            <span className="inline-block overflow-hidden text-ellipsis whitespace-nowrap p-[0_6px]">Login</span>
                        </Link>
                    </div>
                </nav>
            </header>
        </>
    );
}

export default Navbar;
