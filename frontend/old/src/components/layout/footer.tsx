import { Github, Twitter } from "lucide-react";
import Link from "next/link";
import { Separator } from "../ui/separator";

function Footer() {
    return (
        <footer className="mt-4 bg-card px-4 py-12 sm:px-6 lg:px-8">
            <Separator className="my-8" />
            <div className="mx-auto max-w-7xl">
                <div className="grid grid-cols-1 gap-8 md:grid-cols-3">
                    <div className="space-y-4">
                        <h3 className="font-bold text-white text-xl">myrtle.moe</h3>
                        <p className="text-sm">Your go-to resource for all things Arknights.</p>
                        <div className="flex space-x-4">
                            <Link className="transition-colors hover:text-blue-400" href="https://twitter.com" target="_blank">
                                <Twitter className="h-6 w-6" />
                                <span className="sr-only">Twitter</span>
                            </Link>
                            <Link className="transition-colors hover:text-indigo-400" href="https://discord.com" target="_blank">
                                <svg fill="currentColor" height="24" stroke="currentColor" strokeWidth="0" viewBox="0 0 16 16" width="24" xmlns="http://www.w3.org/2000/svg">
                                    <path d="M6.552 6.712c-.456 0-.816.4-.816.888s.368.888.816.888c.456 0 .816-.4.816-.888.008-.488-.36-.888-.816-.888zm2.92 0c-.456 0-.816.4-.816.888s.368.888.816.888c.456 0 .816-.4.816-.888s-.36-.888-.816-.888z"></path>
                                    <path d="M13.36 0H2.64C1.736 0 1 .736 1 1.648v10.816c0 .912.736 1.648 1.64 1.648h9.072l-.424-1.48 1.024.952.968.896L15 16V1.648C15 .736 14.264 0 13.36 0zm-3.088 10.448s-.288-.344-.528-.648c1.048-.296 1.448-.952 1.448-.952-.328.216-.64.368-.92.472-.4.168-.784.28-1.16.344a5.604 5.604 0 0 1-2.072-.008 6.716 6.716 0 0 1-1.176-.344 4.688 4.688 0 0 1-.584-.272c-.024-.016-.048-.024-.072-.04-.016-.008-.024-.016-.032-.024-.144-.08-.224-.136-.224-.136s.384.64 1.4.944c-.24.304-.536.664-.536.664-1.768-.056-2.44-1.216-2.44-1.216 0-2.576 1.152-4.664 1.152-4.664 1.152-.864 2.248-.84 2.248-.84l.08.096c-1.44.416-2.104 1.048-2.104 1.048s.176-.096.472-.232c.856-.376 1.536-.48 1.816-.504.048-.008.088-.016.136-.016a6.521 6.521 0 0 1 4.024.752s-.632-.6-1.992-1.016l.112-.128s1.096-.024 2.248.84c0 0 1.152 2.088 1.152 4.664 0 0-.68 1.16-2.448 1.216z" />
                                </svg>
                                <span className="sr-only">Discord</span>
                            </Link>
                            <Link className="transition-colors hover:text-gray-400" href="https://github.com" target="_blank">
                                <Github className="h-6 w-6" />
                                <span className="sr-only">GitHub</span>
                            </Link>
                        </div>
                    </div>
                    <div>
                        <h4 className="mb-4 font-semibold text-lg text-primary">Quick Links</h4>
                        <ul className="space-y-2">
                            <li>
                                <Link className="transition-colors hover:text-muted-foreground" href="/operators">
                                    Operators
                                </Link>
                            </li>
                            <li>
                                <Link className="transition-colors hover:text-muted-foreground" href="#">
                                    Guides
                                </Link>
                            </li>
                            <li>
                                <Link className="transition-colors hover:text-muted-foreground" href="#">
                                    Events
                                </Link>
                            </li>
                            <li>
                                <Link className="transition-colors hover:text-muted-foreground" href="#">
                                    About Us
                                </Link>
                            </li>
                        </ul>
                    </div>
                    <div>
                        <h4 className="mb-4 font-semibold text-lg text-primary">Resources</h4>
                        <ul className="space-y-2">
                            <li>
                                <Link className="transition-colors hover:text-muted-foreground" href="#">
                                    Tier List
                                </Link>
                            </li>
                            <li>
                                <Link className="transition-colors hover:text-muted-foreground" href="#">
                                    Farming Guide
                                </Link>
                            </li>
                            <li>
                                <Link className="transition-colors hover:text-muted-foreground" href="#">
                                    Latest News
                                </Link>
                            </li>
                            <li>
                                <Link className="transition-colors hover:text-muted-foreground" href="#">
                                    FAQ
                                </Link>
                            </li>
                        </ul>
                    </div>
                </div>
                <Separator className="my-8" />
                <div className="flex flex-col items-center justify-between sm:flex-row">
                    <p className="text-sm">&copy; {new Date().getFullYear()} myrtle.moe. All rights reserved.</p>
                    <div className="mt-4 flex space-x-4 sm:mt-0">
                        <Link className="text-sm transition-colors hover:text-white" href="/privacy">
                            Privacy Policy
                        </Link>
                        <Link className="text-sm transition-colors hover:text-white" href="/terms">
                            Terms of Service
                        </Link>
                    </div>
                </div>
            </div>
        </footer>
    );
}

export default Footer;
