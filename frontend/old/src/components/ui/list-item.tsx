import React from "react";
import { cn } from "~/lib/utils";
import { NavigationMenuLink } from "./navigation-menu";

// eslint-disable-next-line react/display-name
const ListItem = React.forwardRef<React.ElementRef<"a">, React.ComponentPropsWithoutRef<"a">>(({ className, title, children, ...props }, ref) => {
    return (
        <li>
            <NavigationMenuLink asChild>
                <a className={cn("block select-none space-y-1 rounded-md p-3 leading-none no-underline outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground", className)} ref={ref} {...props}>
                    <div className="font-medium text-sm leading-none">{title}</div>
                    <p className="line-clamp-2 text-muted-foreground text-sm leading-snug">{children}</p>
                </a>
            </NavigationMenuLink>
        </li>
    );
});

export { ListItem };
