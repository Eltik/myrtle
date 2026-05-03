import { type Dispatch, type SetStateAction, useEffect, useRef, useState } from "react";
import { cn } from "#/lib/utils";
import type { TabId } from "../UserProfile";
import styles from "./ProfileTabs.module.css";

interface Tab {
    id: string;
    label: string;
    count?: number;
}

interface ProfileTabsProps {
    tabs: Tab[];
    active: string;
    onChange: Dispatch<SetStateAction<TabId>>;
}

export function ProfileTabs({ tabs, active, onChange }: ProfileTabsProps) {
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
