// Format item class type for display
export function formatClassType(classType: string | undefined): string {
    if (!classType) return "Unknown";
    return classType.charAt(0).toUpperCase() + classType.slice(1).toLowerCase();
}

// Format item type for display
export function formatItemType(itemType: string | undefined): string {
    if (!itemType) return "Unknown";
    return itemType
        .split("_")
        .map((word) => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
        .join(" ");
}
