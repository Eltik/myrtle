export function ActiveIndicator() {
    return (
        <span
            className="ml-auto h-4 w-4 pointer-events-none z-20 mt-px text-primary/80 text-xs"
            style={{
                transition: "left 400ms cubic-bezier(0.4, 0, 0.2, 1), opacity 200ms ease-out",
                textShadow: "0 0 8px var(--glow-text-icon)",
            }}
        >
            ◎
        </span>
    );
}
