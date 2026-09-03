import { type Dispatch, type SetStateAction, useEffect, useRef, useState } from "react";
import { cn } from "#/lib/utils";
import type { TabId } from "../types";
import styles from "./ProfileTabs.module.css";

interface ITab {
    id: string;
    label: string;
    count?: number;
}

interface IProfileTabsProps {
    tabs: ITab[];
    active: string;
    onChange: Dispatch<SetStateAction<TabId>>;
}

/**
 * Bottom edge of the profile's sticky stack at >= 640px: the 64px site header
 * plus this tab bar, which MEASURES 44px in the browser (14px padding above and
 * below a 14px line, the 1px border, and a rounding pixel from the badge row).
 * Anything else that sticks on this page (the roster filter panel) must start
 * below it. Kept next to the CSS that produces it.
 */
export const PROFILE_STICKY_OFFSET_PX = 64 + 44;

export function ProfileTabs({ tabs, active, onChange }: IProfileTabsProps) {
    const wrapRef = useRef<HTMLDivElement>(null);
    const [indicator, setIndicator] = useState({ left: 0, width: 0 });

    useEffect(() => {
        if (!wrapRef.current) return;
        const el = wrapRef.current.querySelector<HTMLButtonElement>(`[data-tab="${active}"]`);
        if (!el) return;
        const rect = el.getBoundingClientRect();
        const pRect = wrapRef.current.getBoundingClientRect();
        setIndicator({ left: rect.left - pRect.left, width: rect.width });
    }, [active]);

    return (
        <div className={cn(styles.tabs, "overflow-y-hidden")} role="tablist" aria-label="Profile sections">
            <div className={styles.inner} ref={wrapRef}>
                {tabs.map((tab) => (
                    <button key={tab.id} type="button" role="tab" aria-selected={active === tab.id} data-tab={tab.id} className={cn(styles.tab, active === tab.id && styles.tabActive)} onClick={() => onChange(tab.id as SetStateAction<TabId>)}>
                        {tab.label}
                        {tab.count != null && <span className={cn(styles.count, "tabular-nums")}>{tab.count}</span>}
                    </button>
                ))}
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
