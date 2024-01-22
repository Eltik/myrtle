import Image from "next/image";
import Link from "next/link";
import Navbar from "~/components/navbar";

export default function Home() {
    return (
        <>
            <main className="bg-background">
                <Navbar />
            </main>
        </>
    );
}
