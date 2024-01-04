import Head from "next/head";
import Link from "next/link";
import Navbar from "~/components/navbar";

export default function Home() {
    return (
        <>
            <Navbar active="home" />
        </>
    );
}
