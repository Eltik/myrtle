export function ActiveIndicator() {
    return (
        // Decoration: the row's own `bg-accent` already says which link is current,
        // and a screen reader read the glyph aloud ("Home bullseye").
        <span aria-hidden="true" className="pointer-events-none z-20 mt-px ml-auto h-4 w-4 text-primary/80 text-xs">
            ◎
        </span>
    );
}
