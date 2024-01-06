import Navbar from "~/components/navbar";

export default function Home() {
    return (
        <>
            <main className="bg-main-blue-100">
                <Navbar active="home" />
                <div className="flex h-full items-center justify-center">
                    <h1 className="text-6xl font-bold text-main-blue-500">Hello World!</h1>
                </div>
            </main>
        </>
    );
}
