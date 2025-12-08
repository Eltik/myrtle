import { Github, Twitter } from "lucide-react";
import Link from "next/link";

export function Footer() {
    return (
        <footer className="mt-16 border-border border-t bg-background/50">
            <div className="container mx-auto px-4 py-4">
                <div className="flex items-center justify-between text-muted-foreground text-sm">
                    <p className="text-xs">© 2025 Myrtle</p>
                    <div className="flex items-center gap-4 text-xs">
                        <Link className="transition-colors hover:text-foreground" href="#">
                            About
                        </Link>
                        <Link className="transition-colors hover:text-foreground" href="#">
                            Privacy
                        </Link>
                        <Link className="transition-colors hover:text-foreground" href="#">
                            Terms
                        </Link>
                    </div>
                    <div className="flex items-center gap-3">
                        <Link aria-label="GitHub" className="transition-colors hover:text-foreground" href="#">
                            <Github className="h-4 w-4" />
                        </Link>
                        <Link aria-label="Twitter" className="transition-colors hover:text-foreground" href="#">
                            <Twitter className="h-4 w-4" />
                        </Link>
                    </div>
                </div>
            </div>
        </footer>
    );
}
