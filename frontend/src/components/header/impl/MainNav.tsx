import { Link, useRouterState } from "@tanstack/react-router";
import { Button } from "#/components/ui/button";
import { cn } from "#/lib/utils";
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem } from "#/components/ui/menu";
import { ChevronDown } from "lucide-react";

export interface NavItem {
    href: string;
    label: string;
    items?: NavItem[];
}

interface MainNavProps extends React.ComponentProps<"nav"> {
    items: NavItem[];
}

export function MainNav({ items, className, ...props }: MainNavProps) {
    const router = useRouterState();
    const pathname = router.location.pathname;

    return (
        <nav className={cn("hidden items-center gap-1 md:flex", className)} {...props}>
            {items.map((item) => {
                const isActive = pathname === item.href || (item.href !== "/" && pathname.startsWith(item.href));

                if (item.items) {
                    return (
                        <DropdownMenu key={item.href}>
                            <DropdownMenuTrigger>
                                <Button variant="ghost" size="sm" className={cn("h-8 gap-1 px-3 text-sm font-medium", isActive && "bg-accent text-accent-foreground")}>
                                    {item.label}
                                    <ChevronDown className="h-3 w-3 opacity-50" />
                                </Button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent align="start" className="min-w-48">
                                {item.items.map((subItem) => (
                                    <DropdownMenuItem key={subItem.href} className="p-0">
                                        <Link to={subItem.href} className="flex w-full items-center px-2 py-1.5 no-underline text-foreground">
                                            {subItem.label}
                                        </Link>
                                    </DropdownMenuItem>
                                ))}
                            </DropdownMenuContent>
                        </DropdownMenu>
                    );
                }

                return (
                    <Button key={item.href} variant="ghost" size="sm" className={cn("h-8 px-3 text-sm font-medium", isActive && "bg-accent text-accent-foreground")} render={<Link to={item.href} />}>
                        {item.label}
                    </Button>
                );
            })}
        </nav>
    );
}
