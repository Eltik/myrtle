import Head from "next/head";
import Link from "next/link";
import Navbar from "~/components/navbar";

export default function Home() {
    return (
        <>
            <main className="bg-main-blue-100 h-screen">
                <Navbar active="home" />
            </main>
        </>
    );
}
