export default function Footer() {
    return (
        <footer className="site-footer">
            <div className="page-wrap flex flex-col gap-2.5">
                <div className="inline-flex items-center gap-2.5 font-sans text-sm font-semibold leading-none text-foreground">
                    <span className="brand-bezel small" aria-hidden="true">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round" role="image" aria-label="Logo">
                            <path d="M4 18c2.5 0 2.5-4 5-4s2.5 4 5 4 2.5-4 5-4" />
                            <path d="M4 12c2.5 0 2.5-4 5-4s2.5 4 5 4 2.5-4 5-4" />
                        </svg>
                    </span>
                    <span>
                        myrtle.moe <span className="font-normal text-muted-foreground">· v3</span>
                    </span>
                </div>
                <span className="max-w-130 font-sans text-xs leading-normal text-muted-foreground">Not affiliated with Hypergryph or Yostar. Game data and assets are property of their respective owners.</span>
                <span className="font-mono text-[11px] font-medium leading-none tracking-wide text-muted-foreground opacity-70">built on TanStack Start · COSS UI</span>
            </div>
        </footer>
    );
}
