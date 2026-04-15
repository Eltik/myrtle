import { Link, useRouterState } from "@tanstack/react-router";
import { Button } from "#/components/ui/button";
import { cn } from "#/lib/utils";

export interface NavItem {
    href: string;
    label: string;
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

                return (
                    <Button key={item.href} variant="ghost" size="sm" className={cn("h-8 px-3 text-sm font-medium", isActive && "bg-accent text-accent-foreground")} render={<Link to={item.href} />}>
                        {item.label}
                    </Button>
                );
            })}
        </nav>
    );
}
