/**
 * Chart customization types for the DPS Calculator
 */

/** Chart type options */
export type ChartType = "line" | "area" | "bar";

/** Line interpolation types supported by Recharts */
export type LineType = "monotone" | "linear" | "step" | "stepBefore" | "stepAfter" | "natural" | "basis";

/** Legend position options */
export type LegendPosition = "top" | "bottom" | "left" | "right" | "none";

/** Grid style options */
export type GridStyle = "solid" | "dashed" | "dotted" | "none";

/** Number format options */
export type NumberFormat = "compact" | "standard" | "scientific";

/** Axis scale options */
export type AxisScale = "auto" | "linear" | "log";

/**
 * Visual styling settings for the chart
 */
export interface ChartVisualSettings {
    /** Chart type (line, area, bar) */
    chartType: ChartType;
    /** Line interpolation type */
    lineType: LineType;
    /** Show dots/data points on lines */
    showDots: boolean;
    /** Dot size (radius in pixels) */
    dotSize: number;
    /** Show grid lines */
    showGrid: boolean;
    /** Grid line style */
    gridStyle: GridStyle;
    /** Line stroke width */
    strokeWidth: number;
    /** Legend position */
    legendPosition: LegendPosition;
    /** Show area fill under curves */
    showAreaFill: boolean;
    /** Area fill opacity (0-1) */
    areaFillOpacity: number;
    /** Chart height in pixels */
    chartHeight: number;
    /** Animate chart transitions */
    enableAnimation: boolean;
}

/**
 * Data display settings
 */
export interface ChartDataDisplaySettings {
    /** Show DPS values on chart points (labels) */
    showDataLabels: boolean;
    /** Data label interval (show every Nth point) */
    dataLabelInterval: number;
    /** Show horizontal reference lines */
    showReferenceLines: boolean;
    /** Reference line values (DPS thresholds) */
    referenceLineValues: number[];
    /** Number format for values */
    numberFormat: NumberFormat;
    /** Decimal places for DPS values */
    decimalPlaces: number;
    /** Y-axis scale type */
    yAxisScale: AxisScale;
    /** Show axis labels */
    showAxisLabels: boolean;
    /** Show tooltip on hover */
    showTooltip: boolean;
}

/**
 * Range/calculation settings
 */
export interface ChartRangeSettings {
    /** Minimum DEF value for chart */
    minDef: number;
    /** Maximum DEF value for chart */
    maxDef: number;
    /** DEF step size */
    defStep: number;
    /** Minimum RES value for chart */
    minRes: number;
    /** Maximum RES value for chart */
    maxRes: number;
    /** RES step size */
    resStep: number;
}

/**
 * Export settings
 */
export interface ChartExportSettings {
    /** Include legend in PNG export */
    exportIncludeLegend: boolean;
    /** PNG export scale factor */
    exportScale: number;
    /** PNG background color (transparent or themed) */
    exportBackground: "transparent" | "themed";
}

/**
 * Complete chart settings object
 */
export interface DpsChartSettings {
    visual: ChartVisualSettings;
    dataDisplay: ChartDataDisplaySettings;
    range: ChartRangeSettings;
    export: ChartExportSettings;
}

/**
 * Partial settings for updates
 */
export type PartialChartSettings = {
    visual?: Partial<ChartVisualSettings>;
    dataDisplay?: Partial<ChartDataDisplaySettings>;
    range?: Partial<ChartRangeSettings>;
    export?: Partial<ChartExportSettings>;
};

/** Settings tab type for remembering last opened tab */
export type SettingsTab = "visual" | "data" | "range";
