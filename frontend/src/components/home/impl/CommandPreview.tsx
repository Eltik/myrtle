import { Kbd } from "#/components/ui/kbd";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { useIsMac } from "#/hooks/use-is-mac";
import { ROLE_GRADIENT } from "#/lib/role-styles";
import { cn } from "#/lib/utils";
import styles from "./Hero.module.css";

export default function CommandPreview() {
    const isMac = useIsMac();

    return (
        <div className={styles.cmdPopup}>
            <div className={styles.cmdHead}>
                <svg className={styles.cmdSearchIcon} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" role="image" aria-label="Search">
                    <circle cx="11" cy="11" r="8" />
                    <path d="m21 21-4.35-4.35" />
                </svg>
                <input className={styles.cmdInput} defaultValue="myr" placeholder="Type a command or search…" readOnly suppressHydrationWarning />
                <Kbd>esc</Kbd>
            </div>
            <div className={styles.cmdPanel}>
                <div className={styles.cmdGroup}>
                    <div className={styles.cmdGroupLabel}>Operators</div>
                    <div className={cn(styles.cmdItem, styles.isHighlighted)}>
                        <span className="op-chip" aria-hidden="true" style={{ background: ROLE_GRADIENT.Vanguard }}>
                            <OperatorAvatar charId="char_151_myrtle" name="Myrtle" />
                        </span>
                        <span className={styles.cmdPrimary}>Myrtle</span>
                        <span className={styles.cmdMeta}>4★ · Vanguard · Tactician</span>
                        <Kbd>↵</Kbd>
                    </div>
                    <div className={styles.cmdItem}>
                        <span className="op-chip" aria-hidden="true" style={{ background: ROLE_GRADIENT.Caster }}>
                            <OperatorAvatar charId="char_002_amiya" name="Amiya" />
                        </span>
                        <span className={styles.cmdPrimary}>Amiya</span>
                        <span className={styles.cmdMeta}>6★ · Caster · Core</span>
                    </div>
                    <div className={styles.cmdItem}>
                        <span className="op-chip" aria-hidden="true" style={{ background: ROLE_GRADIENT.Medic }}>
                            <OperatorAvatar charId="char_117_myrrh" name="Myrrh" />
                        </span>
                        <span className={styles.cmdPrimary}>Myrrh</span>
                        <span className={styles.cmdMeta}>4★ · Medic · Medic</span>
                    </div>
                </div>
                <div className={styles.cmdSep} />
                <div className={styles.cmdGroup}>
                    <div className={styles.cmdGroupLabel}>Tools</div>
                    <div className={styles.cmdItem}>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" role="image" aria-label="Calculator">
                            <rect x="3" y="3" width="18" height="18" rx="2" />
                            <path d="M3 9h18M9 21V9" />
                        </svg>
                        <span className={styles.cmdPrimary}>Recruitment calculator</span>
                        <Kbd>{isMac ? "⌘" : "CTRL"}</Kbd>
                        <Kbd>R</Kbd>
                    </div>
                    <div className={styles.cmdItem}>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" role="image" aria-label="Graph">
                            <path d="M3 3v18h18" />
                            <path d="m19 9-5 5-4-4-3 3" />
                        </svg>
                        <span className={styles.cmdPrimary}>DPS charts</span>
                        <Kbd>{isMac ? "⌘" : "CTRL"}</Kbd>
                        <Kbd>D</Kbd>
                    </div>
                </div>
            </div>
            <div className={styles.cmdFooter}>
                <span>
                    <Kbd>↑</Kbd>
                    <Kbd>↓</Kbd> to navigate
                </span>
                <span>
                    <Kbd>↵</Kbd> to select
                </span>
                <span className={styles.right}>powered by COSS UI</span>
            </div>
        </div>
    );
}
