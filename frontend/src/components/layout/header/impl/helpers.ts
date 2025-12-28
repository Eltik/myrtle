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

    // For dropdown items, check if current path matches any dropdown href
    // Also check if pathname matches the base path (e.g., /operators for /operators?id=...)
    return item.dropdown.some((dropdownItem) => {
        const basePath = dropdownItem.href.split("/").slice(0, 2).join("/"); // e.g., "/operators" from "/operators/list"
        return pathname === dropdownItem.href || pathname.startsWith(`${dropdownItem.href}/`) || pathname === basePath;
    });
}
