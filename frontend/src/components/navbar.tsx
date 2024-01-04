import Image from "next/image";
import Link from "next/link";

function Navbar({ active }: { active: "home" | "players" | "statistics" | "login" }) {
    return (
        <>
            <div className="bg-main-blue-200 grid grid-areas-navbar rounded-b-lg" style={{
                gridTemplateColumns: "1fr auto auto auto 1fr",
                gridTemplateRows: "10vh",
                gridColumn: "1/4",
                gridRow: "1/3",
                boxShadow: "0 0 .5em rgb(0, 0, 0)"
            }}>
                <button type="button" className="hidden">
                    <div className="bar1" />
                    <div className="bar2" />
                    <div className="bar3" />
                </button>
                <Image className="ml-5 rounded-sm grid-in-navlogo object-contain h-[min(80%,70px)] cursor-pointer w-auto" src={"/myrtle.webp"} alt="logo" width={100} height={100} loading="lazy" style={{
                    marginBlock: "auto",
                    marginInline: "vw"
                }} />
                <div className="grid-in-navtext flex flex-row justify-center">
                    <Link href={"/"} className="grid-in-navtext cursor-pointer text-main-pink-300" style={{
                        marginBlock: "auto",
                        marginInline: "1.5vw"
                    }}>Home</Link>
                    <Link href={"/players"} className="grid-in-navtext cursor-pointer text-main-pink-300" style={{
                        marginBlock: "auto",
                        marginInline: "1.5vw"
                    }}>Players</Link>
                    <Link href={"/statistics"} className="grid-in-navtext cursor-pointer text-main-pink-300" style={{
                        marginBlock: "auto",
                        marginInline: "1.5vw"
                    }}>Statistics</Link>
                </div>
            </div>
        </>
    );
}

export default Navbar;
