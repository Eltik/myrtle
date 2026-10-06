import { EyeOffIcon } from "lucide-react";
import { type ReactNode, useEffect, useRef, useState } from "react";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { TabId } from "../types";
import type { messages } from "./ProfileTabs.messages";
import styles from "./ProfileTabs.module.css";

interface ITab {
    id: TabId;
    label: string;
    count?: number;
    /** Hidden from visitors. Only the owner is shown such a tab, marked so. */
    private?: boolean;
}

interface IProfileTabsProps {
    tabs: ITab[];
    active: TabId | null;
    onChange: (id: TabId) => void;
    /** Called when a tab is about to be chosen (hover, focus, press), to prefetch it. */
    onIntent?: (id: TabId) => void;
    /** Trailing controls, after the last tab (the owner's Customize button). */
    end?: ReactNode;
}

/**
 * Bottom edge of the profile's sticky stack at >= 640px: the 64px site header
 * plus this tab bar, which MEASURES 44px in the browser (14px padding above and
 * below a 14px line, the 1px border, and a rounding pixel from the badge row).
 * Anything else that sticks on this page (the roster filter panel) must start
 * below it. Kept next to the CSS that produces it.
 */
export const PROFILE_STICKY_OFFSET_PX = 64 + 44;

export function ProfileTabs({ tabs, active, onChange, onIntent, end }: IProfileTabsProps) {
    const t: TypedT<typeof messages> = useT("user");
    const wrapRef = useRef<HTMLDivElement>(null);
    const [indicator, setIndicator] = useState({ left: 0, width: 0 });

    // Re-measured when the order changes too: saving a layout moves the active tab.
    const order = tabs.map((tab) => tab.id).join(" ");
    useEffect(() => {
        if (!wrapRef.current || !order) return;
        const el = wrapRef.current.querySelector<HTMLButtonElement>(`[data-tab="${active}"]`);
        if (!el) return;
        const rect = el.getBoundingClientRect();
        const pRect = wrapRef.current.getBoundingClientRect();
        setIndicator({ left: rect.left - pRect.left, width: rect.width });
    }, [active, order]);

    return (
        <div className={cn(styles.tabs, "overflow-y-hidden")} role="tablist" aria-label={t("profile.tabs.label")}>
            <div className={styles.inner} ref={wrapRef}>
                {tabs.map((tab) => (
                    <button
                        key={tab.id}
                        type="button"
                        role="tab"
                        aria-selected={active === tab.id}
                        data-tab={tab.id}
                        className={cn(styles.tab, active === tab.id && styles.tabActive)}
                        onPointerEnter={() => onIntent?.(tab.id)}
                        onPointerDown={() => onIntent?.(tab.id)}
                        onFocus={() => onIntent?.(tab.id)}
                        onClick={() => onChange(tab.id)}
                    >
                        {tab.private && <EyeOffIcon aria-label={t("profile.tabs.private")} className="size-3.5 opacity-70" />}
                        {tab.label}
                        {tab.count != null && <span className={cn(styles.count, "tabular-nums")}>{tab.count}</span>}
                    </button>
                ))}
                {end && <div className={styles.end}>{end}</div>}
                <span
                    aria-hidden="true"
                    className={styles.indicator}
                    style={{
                        transform: `translateX(${indicator.left}px)`,
                        width: indicator.width,
                    }}
                />
            </div>
        </div>
    );
}
