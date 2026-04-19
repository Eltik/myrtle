export default function Footer() {
    return (
        <footer className="site-footer">
            <div className="footer-inner page-wrap">
                <div className="footer-brand">
                    <span className="brand-bezel small" aria-hidden="true">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round">
                            <path d="M4 18c2.5 0 2.5-4 5-4s2.5 4 5 4 2.5-4 5-4" />
                            <path d="M4 12c2.5 0 2.5-4 5-4s2.5 4 5 4 2.5-4 5-4" />
                        </svg>
                    </span>
                    <span>
                        myrtle.moe <span className="footer-muted">· v3</span>
                    </span>
                </div>
                <span className="footer-note">Not affiliated with Hypergryph or Yostar. Game data and assets are property of their respective owners.</span>
                <span className="footer-meta">built on TanStack Start · COSS UI</span>
            </div>
        </footer>
    );
}
