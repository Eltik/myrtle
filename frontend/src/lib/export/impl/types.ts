export type ExportFormat = "json" | "csv" | "tsv" | "markdown" | "xml" | "yaml";

export type ExportScope = "all" | "filtered" | "page";

export type CsvLineEnding = "lf" | "crlf";

export type JsonIndent = "0" | "2" | "4" | "tab";

export type ArrayJoinMode = "join" | "json";

export interface IExportField<T> {
    /** Stable identifier - used in column selection state and as the default header. */
    id: string;
    /** Human-readable column label (used as the header in CSV/Markdown, and as the key in JSON/XML). */
    label: string;
    /** Optional grouping label rendered in the field-selection UI. */
    group?: string;
    /** Whether the field is enabled by default. */
    defaultEnabled?: boolean;
    /** Pull the value off the source row. Returning `undefined`/`null` yields an empty cell. */
    accessor: (row: T) => unknown;
}

export interface IExportSchema<T> {
    /** Schema identifier (e.g. "operators") - used as the localStorage namespace and default XML element. */
    id: string;
    /** Singular item label for the XML row element & UI copy (e.g. "operator"). */
    itemName: string;
    /** Plural label for UI copy (e.g. "operators"). */
    pluralName: string;
    /** Fields available for export. Order is preserved in the dialog and in output. */
    fields: IExportField<T>[];
}

export interface IExportOptions {
    format: ExportFormat;
    /** Field IDs to include, in the order in which they should appear. */
    fieldIds: string[];
    /** Pretty-print JSON/XML/YAML (indented). */
    pretty: boolean;
    /** JSON indent width when pretty. */
    jsonIndent: JsonIndent;
    /** CSV/TSV delimiter override. Defaults to comma for CSV, tab for TSV. */
    csvDelimiter: string;
    /** Include header row in CSV/TSV. */
    csvHeaders: boolean;
    /** Prepend UTF-8 BOM (helps Excel detect UTF-8 in CSV). */
    csvBom: boolean;
    /** CSV/TSV/Markdown line ending. */
    lineEnding: CsvLineEnding;
    /** How to encode array/object values in flat formats (CSV/TSV/Markdown). */
    arrayMode: ArrayJoinMode;
    /** Separator used when arrayMode is "join". */
    arraySeparator: string;
    /** Markdown: include a numeric row-index column. */
    markdownRowIndex: boolean;
    /** XML root element name. */
    xmlRoot: string;
    /** XML row element name (defaults to schema.itemName). */
    xmlItem: string;
    /** XML: emit primitive fields as attributes on the row element. */
    xmlAttributes: boolean;
}

export interface IExportResult {
    content: string;
    mimeType: string;
    extension: string;
}
