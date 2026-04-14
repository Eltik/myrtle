import type { NavItem } from "./types";

export function isNavItemActive(item: NavItem, pathname: string): boolean {
    // For direct links (no dropdown), check exact match
    if (!item.dropdown) {
        return item.href === pathname;
    }

    // Special case: Users tab should also match /user/[id] profile pages
    if (item.label === "Players" && pathname.startsWith("/user/")) {
        return true;
    }

    // Special case: Collection tab should match operator detail pages
    if (item.label === "Collection" && pathname.startsWith("/collection/operators")) {
        return true;
    }

    // For dropdown items, check if current path matches any dropdown href
    return item.dropdown.some((dropdownItem) => {
        return pathname === dropdownItem.href || pathname.startsWith(`${dropdownItem.href}/`) || pathname.startsWith(`${dropdownItem.href}?`);
    });
}
