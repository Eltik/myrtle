import { useEffect, useRef } from "react";
import { AdminBar } from "./AdminBar";
import { useAdminNav } from "./nav";

/**
 * The admin panel's frame under the site header: the section bar, then the
 * 1320px content column the active section renders into.
 */
export function AdminLayout({ children }: { children: React.ReactNode }): React.ReactElement {
    const nav = useAdminNav();

    // A new section starts at its top, as a new page would. Search-param changes
    // inside a section (filters, selection) keep the scroll position.
    const previous = useRef(nav.current);
    useEffect(() => {
        if (previous.current === nav.current) return;
        previous.current = nav.current;
        window.scrollTo({ top: 0 });
    }, [nav.current]);

    return (
        <div className="bg-background text-foreground">
            <AdminBar nav={nav} />
            <div className="mx-auto max-w-[1320px] px-4 pt-4 pb-10 md:px-8 md:pt-6 md:pb-14 xl:px-10">{children}</div>
        </div>
    );
}
