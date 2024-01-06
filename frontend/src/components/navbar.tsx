import Image from "next/image";
import Link from "next/link";

function Navbar({ active }: { active: "home" | "players" | "statistics" | "login" }) {
    return (
        <>
            <header className="bg-main-blue-200/80 sticky top-0 flex flex-col justify-around w-full p-[0_24px] h-16 backdrop-blur-sm" style={{
                transform: "translateZ(0)",
                boxShadow: "inset 0 -1px 0 0 #333"
            }}>
                <nav className="w-full flex items-center relative" style={{
                    flex: "1 1"
                }}>
                    <div className="md:hidden flex items-center w-full justify-between ">
                        <div className="flex flex-row items-center justify-start gap-1">
                            <Link href={"/"}>
                                <svg aria-label="Vercel logomark" height="22" role="img" className="w-auto overflow-visible" viewBox="0 0 74 64">
                                    <path d="M37.5896 0.25L74.5396 64.25H0.639648L37.5896 0.25Z" fill="#fff"></path>
                                </svg>
                            </Link>
                            <Link href={"/"} target="_blank" className="flex text-white">myrtle.moe</Link>
                        </div>
                        <div className="flex items-center">
                            <button type="button" className="flex p-0 justify-center background-none text-gray-200">
                                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="">
                                    <circle cx="11" cy="11" r="8" />
                                    <path d="m21 21-4.3-4.3" />
                                </svg>
                            </button>
                            <button type="button" className="flex justify-center cursor-pointer m-[0_-5px_0_5px] rounded-md bg-transparent border-none text-gray-200">
                                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="">
                                    <line x1="4" x2="20" y1="12" y2="12" />
                                    <line x1="4" x2="20" y1="6" y2="6" />
                                    <line x1="4" x2="20" y1="18" y2="18" />
                                </svg>
                            </button>
                        </div>
                    </div>
                    <div className="gap-6 w-full hidden md:flex items-center">
                        <div className="flex flex-row items-center justify-start gap-4">
                            <Link href={"/"}>
                                <svg aria-label="Vercel logomark" height="22" role="img" className="w-auto overflow-visible" viewBox="0 0 74 64">
                                    <path d="M37.5896 0.25L74.5396 64.25H0.639648L37.5896 0.25Z" fill="#fff"></path>
                                </svg>
                            </Link>
                            <Link href={"/"} target="_blank" className="flex text-white">myrtle.moe</Link>
                        </div>
                        <Link href={"/players"} className="ml-5 text-main-pink-400 relative transition-all duration-150 ease-in-out hover:text-main-dark-pink-400">Players</Link>
                        <Link href={"/statistics"} className="ml-5 text-main-pink-400 relative transition-all duration-150 ease-in-out hover:text-main-dark-pink-400">Statistics</Link>
                    </div>
                    <div className="gap-2 hidden md:flex items-center">
                        <button type="button" className="flex cursor-pointer items-center justify-between whitespace-nowrap rounded-md bg-main-blue-500 text-gray-50 p-[0_6px_0_8px] h-8 transition-all duration-150 ease-in-out hover:bg-main-blue-500/70">
                            Search players...
                            <kbd className="h-5 p-[0_6px] rounded-md text-gray-50 text-xs ml-4 bg-main-blue-200" style={{
                                lineHeight: "20px",
                                boxShadow: "0 0 0 1px hsla(0,0%,100%,.145)"
                            }}>⌘K</kbd>
                        </button>
                        <Link href={"/login"} className="h-8 rounded-md text-gray-50 bg-main-blue-400 font-bold p-[0_12px] max-w-full justify-center items-center flex relative cursor-pointer outline-none m-0 border-0 transition-all duration-150 ease-in-out hover:bg-main-blue-400/70">
                            <span className="mr-1 flex min-w-5 items-center justify-center">
                                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" stroke-linejoin="round" className="">
                                    <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4" />
                                    <polyline points="10 17 15 12 10 7" />
                                    <line x1="15" x2="3" y1="12" y2="12" />
                                </svg>
                            </span>
                            <span className="whitespace-nowrap text-ellipsis overflow-hidden inline-block p-[0_6px]">Login</span>
                        </Link>
                    </div>
                </nav>
            </header>
        </>
    );
}

export default Navbar;
