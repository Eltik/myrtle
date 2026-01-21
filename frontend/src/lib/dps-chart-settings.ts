/**
 * DPS Chart Settings utilities
 * Handles localStorage persistence and default values
 */

import type { DpsChartSettings, PartialChartSettings, SettingsTab } from "~/types/frontend/impl/tools/dps-chart-settings";

export const DPS_CHART_SETTINGS_STORAGE_KEY = "myrtle-dps-chart-settings";
export const DPS_CHART_SETTINGS_TAB_KEY = "myrtle-dps-chart-settings-tab";

/**
 * Default chart settings - matches current behavior
 */
export const DEFAULT_CHART_SETTINGS: DpsChartSettings = {
    visual: {
        chartType: "line",
        lineType: "monotone",
        showDots: false,
        dotSize: 4,
        showGrid: true,
        gridStyle: "dashed",
        strokeWidth: 2,
        legendPosition: "bottom",
        showAreaFill: false,
        areaFillOpacity: 0.3,
        chartHeight: 400,
        enableAnimation: true,
    },
    dataDisplay: {
        showDataLabels: false,
        dataLabelInterval: 5,
        showReferenceLines: false,
        referenceLineValues: [],
        numberFormat: "compact",
        decimalPlaces: 0,
        yAxisScale: "auto",
        showAxisLabels: true,
        showTooltip: true,
    },
    range: {
        minDef: 0,
        maxDef: 3000,
        defStep: 20,
        minRes: 0,
        maxRes: 120,
        resStep: 10,
    },
    export: {
        exportIncludeLegend: true,
        exportScale: 2,
        exportBackground: "themed",
    },
};

/**
 * Read stored settings from localStorage
 */
export function getStoredChartSettings(): Partial<DpsChartSettings> | null {
    if (typeof window === "undefined") return null;
    try {
        const stored = localStorage.getItem(DPS_CHART_SETTINGS_STORAGE_KEY);
        if (stored === null) return null;
        return JSON.parse(stored) as Partial<DpsChartSettings>;
    } catch {
        return null;
    }
}

/**
 * Save settings to localStorage
 */
export function setStoredChartSettings(settings: DpsChartSettings | null): void {
    if (typeof window === "undefined") return;
    if (settings === null) {
        localStorage.removeItem(DPS_CHART_SETTINGS_STORAGE_KEY);
    } else {
        localStorage.setItem(DPS_CHART_SETTINGS_STORAGE_KEY, JSON.stringify(settings));
    }
}

/**
 * Get stored settings tab
 */
export function getStoredSettingsTab(): SettingsTab {
    if (typeof window === "undefined") return "visual";
    try {
        const stored = localStorage.getItem(DPS_CHART_SETTINGS_TAB_KEY);
        if (stored === "visual" || stored === "data" || stored === "range") {
            return stored;
        }
        return "visual";
    } catch {
        return "visual";
    }
}

/**
 * Save settings tab to localStorage
 */
export function setStoredSettingsTab(tab: SettingsTab): void {
    if (typeof window === "undefined") return;
    localStorage.setItem(DPS_CHART_SETTINGS_TAB_KEY, tab);
}

/**
 * Deep merge chart settings
 */
export function mergeChartSettings(base: DpsChartSettings, updates: PartialChartSettings): DpsChartSettings {
    return {
        visual: { ...base.visual, ...updates.visual },
        dataDisplay: { ...base.dataDisplay, ...updates.dataDisplay },
        range: { ...base.range, ...updates.range },
        export: { ...base.export, ...updates.export },
    };
}

/**
 * Format a number according to the specified format
 */
export function formatChartNumber(value: number, format: "compact" | "standard" | "scientific", decimalPlaces: number): string {
    if (format === "compact") {
        if (value >= 1000000) {
            return `${(value / 1000000).toFixed(decimalPlaces)}M`;
        }
        if (value >= 1000) {
            return `${(value / 1000).toFixed(decimalPlaces)}K`;
        }
        return value.toFixed(decimalPlaces);
    }
    if (format === "scientific") {
        return value.toExponential(decimalPlaces);
    }
    // "standard" format
    return value.toLocaleString(undefined, {
        minimumFractionDigits: decimalPlaces,
        maximumFractionDigits: decimalPlaces,
    });
}

/**
 * Get strokeDasharray value for grid style
 */
export function getGridStrokeDasharray(style: "solid" | "dashed" | "dotted" | "none"): string {
    if (style === "dashed") {
        return "3 3";
    }
    if (style === "dotted") {
        return "1 3";
    }
    // "solid" and "none" both return no dash
    return "0";
}
