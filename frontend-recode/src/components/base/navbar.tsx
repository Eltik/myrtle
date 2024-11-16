import Link from "next/link";
import { Button } from "~/components/ui/button";
import { SidebarTrigger } from "~/components/ui/sidebar";
import { Dialog, DialogTrigger } from "../ui/dialog";
import { LoginDialogue } from "./login-dialogue";

export function Navbar() {
    return (
        <>
            <Dialog>
                <header className="bg-background-900/95 fixed top-0 z-30 w-full border-b shadow backdrop-blur supports-[backdrop-filter]:bg-background/60 dark:shadow-none">
                    <div className="mx-4 flex h-14 items-center sm:mx-4 md:mx-8 md:ml-32 lg:ml-32">
                        <div className="flex items-center">
                            <SidebarTrigger className="block md:hidden" />
                            <Link href={"/"} className="text-base font-bold">
                                myrtle.moe
                            </Link>
                        </div>
                        <div className="flex flex-1 items-center justify-end space-x-2 lg:mr-14">
                            <DialogTrigger asChild>
                                <Button>Login</Button>
                            </DialogTrigger>
                            <LoginDialogue />
                        </div>
                    </div>
                </header>
            </Dialog>
        </>
    );
}
