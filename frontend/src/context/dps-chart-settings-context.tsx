"use client";

import { createContext, type ReactNode, useCallback, useContext, useEffect, useState } from "react";
import type { DpsChartSettings, PartialChartSettings } from "~/components/tools/dps-calculator/impl/chart-settings/types";
import { DEFAULT_CHART_SETTINGS, getStoredChartSettings, mergeChartSettings, setStoredChartSettings } from "~/lib/dps-chart-settings";

interface DpsChartSettingsContextType {
    /** Current chart settings */
    settings: DpsChartSettings;
    /** Whether using default settings */
    isDefault: boolean;
    /** Update specific settings (deep merge) */
    updateSettings: (updates: PartialChartSettings) => void;
    /** Reset all settings to defaults */
    resetToDefaults: () => void;
    /** Reset a specific section to defaults */
    resetSection: (section: keyof DpsChartSettings) => void;
}

const DpsChartSettingsContext = createContext<DpsChartSettingsContextType | undefined>(undefined);

interface DpsChartSettingsProviderProps {
    children: ReactNode;
}

export function DpsChartSettingsProvider({ children }: DpsChartSettingsProviderProps) {
    const [settings, setSettings] = useState<DpsChartSettings>(DEFAULT_CHART_SETTINGS);
    const [isDefault, setIsDefault] = useState(true);
    const [mounted, setMounted] = useState(false);

    // Load stored settings on mount
    useEffect(() => {
        setMounted(true);
        const stored = getStoredChartSettings();
        if (stored !== null) {
            setSettings(mergeChartSettings(DEFAULT_CHART_SETTINGS, stored));
            setIsDefault(false);
        }
    }, []);

    const updateSettings = useCallback((updates: PartialChartSettings) => {
        setSettings((prev) => {
            const merged = mergeChartSettings(prev, updates);
            setStoredChartSettings(merged);
            setIsDefault(false);
            return merged;
        });
    }, []);

    const resetToDefaults = useCallback(() => {
        setSettings(DEFAULT_CHART_SETTINGS);
        setIsDefault(true);
        setStoredChartSettings(null);
    }, []);

    const resetSection = useCallback((section: keyof DpsChartSettings) => {
        setSettings((prev) => {
            const updated = {
                ...prev,
                [section]: DEFAULT_CHART_SETTINGS[section],
            };
            setStoredChartSettings(updated);
            return updated;
        });
    }, []);

    // Avoid hydration mismatch by returning defaults until mounted
    const value: DpsChartSettingsContextType = mounted
        ? { settings, isDefault, updateSettings, resetToDefaults, resetSection }
        : {
              settings: DEFAULT_CHART_SETTINGS,
              isDefault: true,
              updateSettings: () => {},
              resetToDefaults: () => {},
              resetSection: () => {},
          };

    return <DpsChartSettingsContext.Provider value={value}>{children}</DpsChartSettingsContext.Provider>;
}

export function useDpsChartSettings() {
    const context = useContext(DpsChartSettingsContext);
    if (context === undefined) {
        throw new Error("useDpsChartSettings must be used within a DpsChartSettingsProvider");
    }
    return context;
}

/**
 * Hook that safely returns chart settings context or defaults.
 * Use this in components that may render before the provider is mounted.
 */
export function useDpsChartSettingsSafe() {
    const context = useContext(DpsChartSettingsContext);
    return (
        context ?? {
            settings: DEFAULT_CHART_SETTINGS,
            isDefault: true,
            updateSettings: () => {},
            resetToDefaults: () => {},
            resetSection: () => {},
        }
    );
}
