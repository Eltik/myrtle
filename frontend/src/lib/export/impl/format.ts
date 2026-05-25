import type { ArrayJoinMode, IExportField, IExportOptions, IExportResult, IExportSchema } from "./types";

function isPlainObject(value: unknown): value is Record<string, unknown> {
    return typeof value === "object" && value !== null && !Array.isArray(value);
}

function flattenValueForCell(value: unknown, mode: ArrayJoinMode, separator: string): string {
    if (value === null || value === undefined) return "";
    if (typeof value === "string") return value;
    if (typeof value === "number" || typeof value === "boolean") return String(value);
    if (Array.isArray(value)) {
        if (mode === "json") return JSON.stringify(value);
        return value.map((v) => flattenValueForCell(v, mode, separator)).join(separator);
    }
    if (isPlainObject(value)) {
        return JSON.stringify(value);
    }
    return String(value);
}

function pickFields<T>(schema: IExportSchema<T>, fieldIds: string[]): IExportField<T>[] {
    const map = new Map<string, IExportField<T>>(schema.fields.map((f) => [f.id, f]));
    const ordered: IExportField<T>[] = [];
    for (const id of fieldIds) {
        const f = map.get(id);
        if (f) ordered.push(f);
    }
    return ordered;
}

function eol(options: IExportOptions): string {
    return options.lineEnding === "crlf" ? "\r\n" : "\n";
}

function escapeCsvCell(value: string, delimiter: string): string {
    if (value === "") return "";
    const mustQuote = value.includes(delimiter) || value.includes('"') || value.includes("\n") || value.includes("\r");
    if (!mustQuote) return value;
    return `"${value.replace(/"/g, '""')}"`;
}

function escapeMarkdownCell(value: string): string {
    // Replace pipes (table separator) and newlines so cells stay on one line.
    return value.replace(/\|/g, "\\|").replace(/\r?\n/g, " ");
}

function escapeXmlText(value: string): string {
    return value.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

function escapeXmlAttr(value: string): string {
    return escapeXmlText(value).replace(/"/g, "&quot;");
}

function sanitizeXmlName(name: string, fallback: string): string {
    let safe = name.replace(/[^A-Za-z0-9_.-]/g, "_");
    if (!/^[A-Za-z_]/.test(safe)) safe = `_${safe}`;
    return safe || fallback;
}

function jsonIndentSpaces(indent: IExportOptions["jsonIndent"]): number | string {
    if (indent === "tab") return "\t";
    if (indent === "4") return 4;
    if (indent === "2") return 2;
    return 0;
}

function buildRow<T>(row: T, fields: IExportField<T>[]): Record<string, unknown> {
    const out: Record<string, unknown> = {};
    for (const f of fields) {
        out[f.label] = f.accessor(row);
    }
    return out;
}

function buildJson<T>(rows: T[], fields: IExportField<T>[], options: IExportOptions): IExportResult {
    const objects = rows.map((row) => buildRow(row, fields));
    const indent = options.pretty ? jsonIndentSpaces(options.jsonIndent) : 0;
    return {
        content: JSON.stringify(objects, null, indent),
        mimeType: "application/json",
        extension: "json",
    };
}

function buildDelimited<T>(rows: T[], fields: IExportField<T>[], options: IExportOptions, delimiter: string, ext: string, mimeType: string): IExportResult {
    const nl = eol(options);
    const lines: string[] = [];
    if (options.csvHeaders) {
        lines.push(fields.map((f) => escapeCsvCell(f.label, delimiter)).join(delimiter));
    }
    for (const row of rows) {
        const cells = fields.map((f) => escapeCsvCell(flattenValueForCell(f.accessor(row), options.arrayMode, options.arraySeparator), delimiter));
        lines.push(cells.join(delimiter));
    }
    const body = lines.join(nl);
    const content = options.csvBom ? `﻿${body}` : body;
    return { content, mimeType, extension: ext };
}

function buildCsv<T>(rows: T[], fields: IExportField<T>[], options: IExportOptions): IExportResult {
    const delim = options.csvDelimiter || ",";
    return buildDelimited(rows, fields, options, delim, "csv", "text/csv;charset=utf-8");
}

function buildTsv<T>(rows: T[], fields: IExportField<T>[], options: IExportOptions): IExportResult {
    const delim = options.csvDelimiter && options.csvDelimiter !== "," ? options.csvDelimiter : "\t";
    return buildDelimited(rows, fields, options, delim, "tsv", "text/tab-separated-values;charset=utf-8");
}

function buildMarkdown<T>(rows: T[], fields: IExportField<T>[], options: IExportOptions): IExportResult {
    const nl = eol(options);
    const cols: string[] = [];
    if (options.markdownRowIndex) cols.push("#");
    for (const f of fields) cols.push(f.label);
    const header = `| ${cols.map(escapeMarkdownCell).join(" | ")} |`;
    const separator = `| ${cols.map(() => "---").join(" | ")} |`;
    const body = rows.map((row, i) => {
        const cells: string[] = [];
        if (options.markdownRowIndex) cells.push(String(i + 1));
        for (const f of fields) {
            cells.push(escapeMarkdownCell(flattenValueForCell(f.accessor(row), options.arrayMode, options.arraySeparator)));
        }
        return `| ${cells.join(" | ")} |`;
    });
    const content = [header, separator, ...body].join(nl);
    return { content, mimeType: "text/markdown;charset=utf-8", extension: "md" };
}

function valueToXml(value: unknown, key: string, indent: string, pretty: boolean, level: number): string {
    const tag = sanitizeXmlName(key, "item");
    const pad = pretty ? indent.repeat(level) : "";
    const nl = pretty ? "\n" : "";
    if (value === null || value === undefined) return `${pad}<${tag}/>`;
    if (Array.isArray(value)) {
        if (value.length === 0) return `${pad}<${tag}/>`;
        const inner = value.map((v) => valueToXml(v, "item", indent, pretty, level + 1)).join(nl);
        return `${pad}<${tag}>${nl}${inner}${nl}${pad}</${tag}>`;
    }
    if (isPlainObject(value)) {
        const entries = Object.entries(value);
        if (entries.length === 0) return `${pad}<${tag}/>`;
        const inner = entries.map(([k, v]) => valueToXml(v, k, indent, pretty, level + 1)).join(nl);
        return `${pad}<${tag}>${nl}${inner}${nl}${pad}</${tag}>`;
    }
    return `${pad}<${tag}>${escapeXmlText(String(value))}</${tag}>`;
}

function buildXml<T>(rows: T[], fields: IExportField<T>[], options: IExportOptions, fallbackItem: string): IExportResult {
    const indent = "    ";
    const pretty = options.pretty;
    const nl = pretty ? "\n" : "";
    const rootName = sanitizeXmlName(options.xmlRoot || "items", "items");
    const itemName = sanitizeXmlName(options.xmlItem || fallbackItem, fallbackItem);
    const items = rows.map((row) => {
        const attrs: string[] = [];
        const children: string[] = [];
        for (const f of fields) {
            const value = f.accessor(row);
            const tag = sanitizeXmlName(f.label, "field");
            if (options.xmlAttributes && (typeof value === "string" || typeof value === "number" || typeof value === "boolean")) {
                attrs.push(`${tag}="${escapeXmlAttr(String(value))}"`);
            } else if (value === null || value === undefined) {
                if (options.xmlAttributes) attrs.push(`${tag}=""`);
                else children.push(`${pretty ? indent.repeat(2) : ""}<${tag}/>`);
            } else {
                children.push(valueToXml(value, f.label, indent, pretty, 2));
            }
        }
        const open = attrs.length > 0 ? `<${itemName} ${attrs.join(" ")}>` : `<${itemName}>`;
        const pad = pretty ? indent : "";
        if (children.length === 0) {
            const selfClose = attrs.length > 0 ? `<${itemName} ${attrs.join(" ")}/>` : `<${itemName}/>`;
            return `${pad}${selfClose}`;
        }
        return `${pad}${open}${nl}${children.join(nl)}${nl}${pad}</${itemName}>`;
    });
    const header = `<?xml version="1.0" encoding="UTF-8"?>${nl}<${rootName}>`;
    const footer = `</${rootName}>`;
    const content = `${header}${nl}${items.join(nl)}${nl}${footer}`;
    return { content, mimeType: "application/xml;charset=utf-8", extension: "xml" };
}

const YAML_BARE_RE = /^[A-Za-z0-9_][A-Za-z0-9_\-./]*$/;
const YAML_SPECIAL = new Set(["true", "false", "null", "yes", "no", "on", "off", "~"]);

function yamlQuoteString(value: string): string {
    if (value === "") return '""';
    if (/^-?\d+(\.\d+)?$/.test(value)) return `"${value}"`;
    if (YAML_SPECIAL.has(value.toLowerCase())) return `"${value}"`;
    if (YAML_BARE_RE.test(value) && !value.startsWith("-") && !value.startsWith("?")) return value;
    return `"${value.replace(/\\/g, "\\\\").replace(/"/g, '\\"').replace(/\n/g, "\\n").replace(/\r/g, "\\r").replace(/\t/g, "\\t")}"`;
}

function yamlKey(key: string): string {
    if (YAML_BARE_RE.test(key)) return key;
    return `"${key.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`;
}

function valueToYaml(value: unknown, level: number, indentUnit: string): string {
    const indent = indentUnit.repeat(level);
    if (value === null || value === undefined) return "null";
    if (typeof value === "boolean") return value ? "true" : "false";
    if (typeof value === "number") return Number.isFinite(value) ? String(value) : "null";
    if (typeof value === "string") return yamlQuoteString(value);
    if (Array.isArray(value)) {
        if (value.length === 0) return "[]";
        return value
            .map((v) => {
                if (isPlainObject(v) || Array.isArray(v)) {
                    const block = valueToYaml(v, level + 1, indentUnit);
                    if (block.startsWith("\n")) return `${indent}-${block.replace(/^\n/, "\n")}`;
                    return `${indent}-\n${block}`;
                }
                return `${indent}- ${valueToYaml(v, level + 1, indentUnit)}`;
            })
            .join("\n");
    }
    if (isPlainObject(value)) {
        const entries = Object.entries(value);
        if (entries.length === 0) return "{}";
        const lines = entries.map(([k, v]) => {
            const key = yamlKey(k);
            if (v === null || v === undefined) return `${indent}${key}: null`;
            if (Array.isArray(v) || isPlainObject(v)) {
                const block = valueToYaml(v, level + 1, indentUnit);
                if (Array.isArray(v) && v.length === 0) return `${indent}${key}: []`;
                if (isPlainObject(v) && Object.keys(v).length === 0) return `${indent}${key}: {}`;
                return `${indent}${key}:\n${block}`;
            }
            return `${indent}${key}: ${valueToYaml(v, level + 1, indentUnit)}`;
        });
        return lines.join("\n");
    }
    return yamlQuoteString(String(value));
}

function buildYaml<T>(rows: T[], fields: IExportField<T>[], options: IExportOptions): IExportResult {
    const indentUnit = options.jsonIndent === "tab" ? "\t" : options.jsonIndent === "4" ? "    " : "  ";
    const objects = rows.map((row) => buildRow(row, fields));
    const content = valueToYaml(objects, 0, indentUnit);
    return { content, mimeType: "text/yaml;charset=utf-8", extension: "yaml" };
}

export function buildExport<T>(rows: T[], schema: IExportSchema<T>, options: IExportOptions): IExportResult {
    const fields = pickFields(schema, options.fieldIds);
    switch (options.format) {
        case "json":
            return buildJson(rows, fields, options);
        case "csv":
            return buildCsv(rows, fields, options);
        case "tsv":
            return buildTsv(rows, fields, options);
        case "markdown":
            return buildMarkdown(rows, fields, options);
        case "xml":
            return buildXml(rows, fields, options, schema.itemName);
        case "yaml":
            return buildYaml(rows, fields, options);
    }
}
