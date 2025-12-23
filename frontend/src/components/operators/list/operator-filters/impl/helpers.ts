// Helper to create toggle functions for array state
export function createToggle<T>(selected: T[], onChange: (items: T[]) => void) {
    return (item: T) => {
        if (selected.includes(item)) {
            onChange(selected.filter((i) => i !== item));
        } else {
            onChange([...selected, item]);
        }
    };
}
