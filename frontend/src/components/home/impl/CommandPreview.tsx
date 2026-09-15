import { Link } from "@tanstack/react-router";
import { Kbd } from "#/components/ui/kbd";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { ToolIcon } from "#/lib/registry/ToolIcon";
import { TOOLS } from "#/lib/registry/tools";
import { ROLE_GRADIENT, type Role } from "#/lib/role-styles";
import type { messages } from "./CommandPreview.messages";
import styles from "./Hero.module.css";

interface IPreviewOperator {
    id: string;
    name: string;
    meta: string;
    role: Role;
}

const PREVIEW_OPERATORS: IPreviewOperator[] = [
    { id: "char_151_myrtle", name: "Myrtle", meta: "4★ · Vanguard · Flagbearer", role: "Vanguard" },
    { id: "char_242_otter", name: "Mayer", meta: "5★ · Supporter · Summoner", role: "Supporter" },
    { id: "char_117_myrrh", name: "Myrrh", meta: "4★ · Medic · Medic", role: "Medic" },
];

const PREVIEW_TOOL_IDS = ["recruitment", "dps"] as const;

interface ICommandPreviewProps {
    onOpenCommand: () => void;
}

export default function CommandPreview({ onOpenCommand }: ICommandPreviewProps) {
    const t: TypedT<typeof messages> = useT("home");
    // Tool labels live in the `nav` namespace with the rest of the registry copy.
    const tNav = useT("nav");
    const toolsById = new Map(TOOLS.map((t) => [t.id, t]));
    const tools = PREVIEW_TOOL_IDS.map((id) => toolsById.get(id)).filter((t): t is NonNullable<typeof t> => Boolean(t));

    const stop = (e: React.MouseEvent) => e.stopPropagation();

    return (
        <div className={styles.cmdPopup}>
            <button type="button" className={styles.cmdHead} onClick={onOpenCommand} aria-label={t("cmd.open")}>
                <svg className={styles.cmdSearchIcon} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" role="image" aria-label={t("cmd.searchIcon")}>
                    <circle cx="11" cy="11" r="8" />
                    <path d="m21 21-4.35-4.35" />
                </svg>
                <span className={styles.cmdInputFake} suppressHydrationWarning>
                    myr
                </span>
                <Kbd>esc</Kbd>
            </button>
            {/* biome-ignore lint/a11y/useKeyWithClickEvents: clickable fallback background; items handle their own keyboard focus */}
            {/* biome-ignore lint/a11y/noStaticElementInteractions: same */}
            <div className={styles.cmdPanel} onClick={onOpenCommand}>
                <div className={styles.cmdGroup} role="menu">
                    <div className={styles.cmdGroupLabel}>{t("cmd.group.operators")}</div>
                    {PREVIEW_OPERATORS.map((op) => (
                        <Link key={op.id} to="/operators/$id" params={{ id: op.id }} className={styles.cmdItem} role="menuitem" onClick={stop}>
                            <span className="op-chip" aria-hidden="true" style={{ background: ROLE_GRADIENT[op.role] }}>
                                <OperatorAvatar charId={op.id} name={op.name} />
                            </span>
                            <span className={styles.cmdPrimary}>{op.name}</span>
                            <span className={styles.cmdMeta}>{op.meta}</span>
                        </Link>
                    ))}
                </div>
                <div className={styles.cmdSep} />
                <div className={styles.cmdGroup} role="menu">
                    <div className={styles.cmdGroupLabel}>{t("cmd.group.tools")}</div>
                    {tools.map((tool) => (
                        <Link key={tool.id} to={tool.href} className={styles.cmdItem} role="menuitem" onClick={stop}>
                            <ToolIcon name={tool.icon} />
                            <span className={styles.cmdPrimary}>{tNav(tool.labelKey)}</span>
                        </Link>
                    ))}
                </div>
            </div>
            <div className={styles.cmdFooter}>
                <span>
                    <Kbd>↑</Kbd>
                    <Kbd>↓</Kbd> {t("cmd.toNavigate")}
                </span>
                <span>
                    <Kbd>↵</Kbd> {t("cmd.toSelect")}
                </span>
                <span className={styles.right}>{t("cmd.poweredBy")}</span>
            </div>
        </div>
    );
}
