export default function Footer() {
    const year = new Date().getFullYear();

    return (
        <footer className="mt-20 border-t border-[var(--line)] px-4 pb-14 pt-10 text-[var(--sea-ink-soft)]">
            <p>&copy; {year} myrtle.moe</p>
        </footer>
    );
}
