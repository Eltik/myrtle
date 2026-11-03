export function ActiveIndicator() {
    return (
        // Decoration: the row's own `bg-accent` already says which link is current,
        // and a screen reader read the glyph aloud ("Home bullseye").
        <span
            aria-hidden="true"
            className="pointer-events-none z-20 mt-px ml-auto h-4 w-4 text-primary/80 text-xs"
            style={{
                transition: "opacity 200ms ease-out",
                // `--glow-text-icon` is set only once a visitor picks an accent (the theme
                // init script and `applyAccentColors`); with the default accent it is unset
                // and this shadow paints nothing.
                textShadow: "0 0 8px var(--glow-text-icon)",
            }}
        >
            ◎
        </span>
    );
}
