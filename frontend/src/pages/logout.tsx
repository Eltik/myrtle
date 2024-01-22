import { useEffect } from "react";
import Navbar from "~/components/navbar";
import { useLogin, usePlayer } from "~/store/store";

export default function Logout() {
    useEffect(() => {
        useLogin.setState({ loginData: null });
        usePlayer.setState({ playerData: {} });

        window.location.href = "/";
    });

    return (
        <>
            <main className="bg-main-blue-200">
                <Navbar />
            </main>
        </>
    );
}
