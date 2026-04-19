import { Kbd } from "#/components/ui/kbd";
import { OperatorAvatar } from "#/components/ui/operator-avatar";

export default function CommandPreview() {
    return (
        <div className="cmd-popup">
            <div className="cmd-head">
                <svg className="cmd-search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <circle cx="11" cy="11" r="8" />
                    <path d="m21 21-4.35-4.35" />
                </svg>
                <input className="cmd-input" defaultValue="myr" placeholder="Type a command or search…" readOnly suppressHydrationWarning />
                <Kbd>esc</Kbd>
            </div>
            <div className="cmd-panel">
                <div className="cmd-group">
                    <div className="cmd-group-label">Operators</div>
                    <div className="cmd-item is-highlighted">
                        <span className="op-chip c-vanguard" aria-hidden="true">
                            <OperatorAvatar charId="char_151_myrtle" name="Myrtle" />
                        </span>
                        <span className="cmd-primary">Myrtle</span>
                        <span className="cmd-meta">4★ · Vanguard · Tactician</span>
                        <Kbd>↵</Kbd>
                    </div>
                    <div className="cmd-item">
                        <span className="op-chip c-caster" aria-hidden="true">
                            <OperatorAvatar charId="char_213_mostma" name="Mostima" />
                        </span>
                        <span className="cmd-primary">Mostima</span>
                        <span className="cmd-meta">6★ · Caster · Mystic</span>
                    </div>
                    <div className="cmd-item">
                        <span className="op-chip c-medic" aria-hidden="true">
                            <OperatorAvatar charId="char_117_myrrh" name="Myrrh" />
                        </span>
                        <span className="cmd-primary">Myrrh</span>
                        <span className="cmd-meta">4★ · Medic · Medic</span>
                    </div>
                </div>
                <div className="cmd-sep" />
                <div className="cmd-group">
                    <div className="cmd-group-label">Tools</div>
                    <div className="cmd-item">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <rect x="3" y="3" width="18" height="18" rx="2" />
                            <path d="M3 9h18M9 21V9" />
                        </svg>
                        <span className="cmd-primary">Recruitment calculator</span>
                        <Kbd>⌘</Kbd>
                        <Kbd>R</Kbd>
                    </div>
                    <div className="cmd-item">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <path d="M3 3v18h18" />
                            <path d="m19 9-5 5-4-4-3 3" />
                        </svg>
                        <span className="cmd-primary">DPS charts</span>
                        <Kbd>⌘</Kbd>
                        <Kbd>D</Kbd>
                    </div>
                </div>
            </div>
            <div className="cmd-footer">
                <span>
                    <Kbd>↑</Kbd>
                    <Kbd>↓</Kbd> to navigate
                </span>
                <span>
                    <Kbd>↵</Kbd> to select
                </span>
                <span className="right">powered by COSS UI</span>
            </div>
        </div>
    );
}
