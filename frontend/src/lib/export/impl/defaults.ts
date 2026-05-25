import type { IExportOptions, IExportSchema } from "./types";

export function defaultExportOptions<T>(schema: IExportSchema<T>): IExportOptions {
    return {
        format: "json",
        fieldIds: schema.fields.filter((f) => f.defaultEnabled).map((f) => f.id),
        pretty: true,
        jsonIndent: "2",
        csvDelimiter: ",",
        csvHeaders: true,
        csvBom: false,
        lineEnding: "lf",
        arrayMode: "join",
        arraySeparator: "; ",
        markdownRowIndex: false,
        xmlRoot: schema.pluralName,
        xmlItem: schema.itemName,
        xmlAttributes: false,
    };
}

export function timestampedFilename(base: string, format: string): string {
    const now = new Date();
    const pad = (n: number) => n.toString().padStart(2, "0");
    const stamp = `${now.getFullYear()}${pad(now.getMonth() + 1)}${pad(now.getDate())}-${pad(now.getHours())}${pad(now.getMinutes())}`;
    return `${base}-${stamp}.${format}`;
}
