"use client";

import { CheckIcon, CopyIcon, DownloadIcon, FileTextIcon } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { Button } from "#/components/ui/button";
import { Checkbox } from "#/components/ui/checkbox";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Input } from "#/components/ui/input";
import { RadioGroup, RadioGroupItem } from "#/components/ui/radio-group";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Switch } from "#/components/ui/switch";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { buildExport, copyToClipboard, defaultExportOptions, downloadExport, type ExportFormat, type ExportScope, type IExportOptions, type IExportSchema, timestampedFilename } from "#/lib/export";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./ExportDialog.messages";

interface IExportDialogProps<T> {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    schema: IExportSchema<T>;
    /** Full dataset (used when scope = "all"). */
    allRows: T[];
    /** Filtered dataset (used when scope = "filtered"). Defaults to allRows. */
    filteredRows?: T[];
    /** Currently visible page (used when scope = "page"). Optional. */
    pageRows?: T[];
    /** Display label used in the dialog header (e.g. "Operators"). */
    title?: string;
}

type ExportMessageKey = keyof typeof messages & string;
type ExportT = TypedT<typeof messages>;

const FORMAT_LABELS: Record<ExportFormat, { labelKey: ExportMessageKey; descKey: ExportMessageKey }> = {
    json: { labelKey: "export.format.json.label", descKey: "export.format.json.desc" },
    csv: { labelKey: "export.format.csv.label", descKey: "export.format.csv.desc" },
    tsv: { labelKey: "export.format.tsv.label", descKey: "export.format.tsv.desc" },
    markdown: { labelKey: "export.format.markdown.label", descKey: "export.format.markdown.desc" },
    xml: { labelKey: "export.format.xml.label", descKey: "export.format.xml.desc" },
    yaml: { labelKey: "export.format.yaml.label", descKey: "export.format.yaml.desc" },
};

const FORMAT_ORDER: ExportFormat[] = ["json", "csv", "tsv", "markdown", "xml", "yaml"];

const SCOPE_LABEL_KEYS: Record<ExportScope, ExportMessageKey> = {
    all: "export.scope.all",
    filtered: "export.scope.filtered",
    page: "export.scope.page",
};

const PREVIEW_LIMIT = 4000;
const PREVIEW_ROW_LIMIT = 10;

export function ExportDialog<T>({ open, onOpenChange, schema, allRows, filteredRows, pageRows, title }: IExportDialogProps<T>): React.ReactElement {
    const t: ExportT = useT("skins");
    const fmt = useFormatters();
    const storageKey = `myrtle:export:${schema.id}:v1`;
    const [options, setOptions] = useLocalStorageState<IExportOptions>(storageKey, defaultExportOptions(schema), {
        parse: (raw) => {
            try {
                const parsed = JSON.parse(raw) as Partial<IExportOptions>;
                const merged: IExportOptions = { ...defaultExportOptions(schema), ...parsed };
                // Re-validate fieldIds against the current schema in case fields changed across versions.
                const valid = new Set(schema.fields.map((f) => f.id));
                merged.fieldIds = merged.fieldIds.filter((id) => valid.has(id));
                if (merged.fieldIds.length === 0) merged.fieldIds = defaultExportOptions(schema).fieldIds;
                return merged;
            } catch {
                return undefined;
            }
        },
        serialize: (v) => JSON.stringify(v),
    });

    const [scope, setScope] = useState<ExportScope>(filteredRows && allRows.length !== filteredRows.length ? "filtered" : "all");
    const [filename, setFilename] = useState<string>(schema.pluralName);
    const [copied, setCopied] = useState(false);

    // Re-sync the default scope when the dialog reopens.
    useEffect(() => {
        if (!open) return;
        setCopied(false);
    }, [open]);

    const sourceRows = useMemo(() => {
        if (scope === "all") return allRows;
        if (scope === "filtered") return filteredRows ?? allRows;
        return pageRows ?? filteredRows ?? allRows;
    }, [scope, allRows, filteredRows, pageRows]);

    const result = useMemo(() => {
        if (options.fieldIds.length === 0) return null;
        try {
            return buildExport(sourceRows, schema, options);
        } catch (err) {
            console.error("Export failed", err);
            return null;
        }
    }, [sourceRows, schema, options]);

    const previewText = useMemo(() => {
        if (!result) return "";
        // Build a short preview using only the first N rows so we don't render a megabyte string into the DOM.
        if (sourceRows.length <= PREVIEW_ROW_LIMIT) {
            return result.content.length > PREVIEW_LIMIT ? `${result.content.slice(0, PREVIEW_LIMIT)}\n…` : result.content;
        }
        try {
            const small = buildExport(sourceRows.slice(0, PREVIEW_ROW_LIMIT), schema, options);
            return small.content.length > PREVIEW_LIMIT ? `${small.content.slice(0, PREVIEW_LIMIT)}\n…` : `${small.content}\n…`;
        } catch {
            return result.content.slice(0, PREVIEW_LIMIT);
        }
    }, [result, sourceRows, schema, options]);

    const bytes = useMemo(() => (result ? new Blob([result.content]).size : 0), [result]);

    const fieldsByGroup = useMemo(() => {
        const groups = new Map<string, typeof schema.fields>();
        for (const f of schema.fields) {
            const key = f.group ?? t("export.fields.general");
            const list = groups.get(key);
            if (list) list.push(f);
            else groups.set(key, [f]);
        }
        return Array.from(groups.entries());
    }, [schema, t]);

    const enabledSet = useMemo(() => new Set(options.fieldIds), [options.fieldIds]);

    function setOption<K extends keyof IExportOptions>(key: K, value: IExportOptions[K]): void {
        setOptions((prev) => ({ ...prev, [key]: value }));
    }

    function toggleField(id: string): void {
        setOptions((prev) => {
            if (prev.fieldIds.includes(id)) {
                return { ...prev, fieldIds: prev.fieldIds.filter((f) => f !== id) };
            }
            // Preserve schema order when re-adding.
            const order = schema.fields.map((f) => f.id);
            const next = [...prev.fieldIds, id];
            next.sort((a, b) => order.indexOf(a) - order.indexOf(b));
            return { ...prev, fieldIds: next };
        });
    }

    function selectAllFields(): void {
        setOptions((prev) => ({ ...prev, fieldIds: schema.fields.map((f) => f.id) }));
    }

    function selectDefaultFields(): void {
        setOptions((prev) => ({ ...prev, fieldIds: schema.fields.filter((f) => f.defaultEnabled).map((f) => f.id) }));
    }

    function clearAllFields(): void {
        setOptions((prev) => ({ ...prev, fieldIds: [] }));
    }

    function handleDownload(): void {
        if (!result) return;
        const base = filename.trim() || schema.pluralName;
        downloadExport(timestampedFilename(base, result.extension), result);
    }

    async function handleCopy(): Promise<void> {
        if (!result) return;
        const ok = await copyToClipboard(result.content);
        if (ok) {
            setCopied(true);
            window.setTimeout(() => setCopied(false), 1500);
        }
    }

    const headline = title ?? schema.pluralName;
    const isFlatFormat = options.format === "csv" || options.format === "tsv" || options.format === "markdown";
    const supportsPretty = options.format === "json" || options.format === "xml";
    const supportsLineEnding = isFlatFormat;
    const supportsIndent = options.format === "json" || options.format === "yaml";

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogPopup className="flex h-[min(820px,92vh)] w-full max-w-260 flex-col overflow-hidden p-0">
                <DialogHeader className="border-b px-6 py-4">
                    <div className="flex items-center gap-3">
                        <div className="grid size-9 place-items-center rounded-md bg-primary/10 text-primary">
                            <FileTextIcon className="size-4.5" />
                        </div>
                        <div className="flex-1">
                            <DialogTitle>{t("export.title", { name: headline })}</DialogTitle>
                            <DialogDescription>{t("export.description")}</DialogDescription>
                        </div>
                    </div>
                </DialogHeader>

                <div className="grid min-h-0 flex-1 grid-cols-1 overflow-hidden lg:grid-cols-[1fr_minmax(0,1.1fr)]">
                    <div className="flex min-h-0 flex-col overflow-y-auto px-6 py-5">
                        <section className="flex flex-col gap-3">
                            <h3 className="font-sans font-semibold text-[13px] text-foreground uppercase tracking-[0.08em]">{t("export.section.format")}</h3>
                            <div className="grid grid-cols-2 gap-1.5 sm:grid-cols-3">
                                {FORMAT_ORDER.map((fmt) => {
                                    const meta = FORMAT_LABELS[fmt];
                                    const selected = options.format === fmt;
                                    return (
                                        <button
                                            key={fmt}
                                            type="button"
                                            onClick={() => setOption("format", fmt)}
                                            className={cn("flex cursor-pointer flex-col items-start gap-0.5 rounded-md border border-border bg-background px-3 py-2 text-left transition-colors hover:border-primary/50 hover:bg-accent/50", selected && "border-primary bg-primary/5 ring-1 ring-primary/30")}
                                            aria-pressed={selected}
                                        >
                                            <span className="font-medium font-sans text-[13px] text-foreground leading-tight">{t(meta.labelKey)}</span>
                                            <span className="font-sans text-[11px] text-muted-foreground leading-snug">{t(meta.descKey)}</span>
                                        </button>
                                    );
                                })}
                            </div>
                        </section>

                        <section className="mt-6 flex flex-col gap-3">
                            <h3 className="font-sans font-semibold text-[13px] text-foreground uppercase tracking-[0.08em]">{t("export.section.scope")}</h3>
                            <RadioGroup className="flex flex-col gap-1.5" value={scope} onValueChange={(v) => setScope(v as ExportScope)}>
                                {(Object.keys(SCOPE_LABEL_KEYS) as ExportScope[]).map((s) => {
                                    const count = s === "all" ? allRows.length : s === "filtered" ? (filteredRows ?? allRows).length : (pageRows?.length ?? 0);
                                    const disabled = s === "page" && (pageRows === undefined || pageRows.length === 0);
                                    return (
                                        // biome-ignore lint/a11y/noLabelWithoutControl: RadioGroupItem renders the input
                                        <label key={s} className={cn("flex cursor-pointer items-center gap-2.5 rounded-md border border-transparent px-2 py-1.5 hover:bg-accent/40", disabled && "cursor-not-allowed opacity-50 hover:bg-transparent")}>
                                            <RadioGroupItem value={s} disabled={disabled} />
                                            <span className="flex flex-1 items-baseline justify-between gap-2">
                                                <span className="font-sans text-[13px] text-foreground leading-none">{t(SCOPE_LABEL_KEYS[s])}</span>
                                                <span className="font-mono text-[11px] text-muted-foreground tabular-nums leading-none">
                                                    {fmt.number(count)} {count === 1 ? schema.itemName : schema.pluralName}
                                                </span>
                                            </span>
                                        </label>
                                    );
                                })}
                            </RadioGroup>
                        </section>

                        <section className="mt-6 flex flex-col gap-3">
                            <div className="flex items-center justify-between gap-2">
                                <h3 className="font-sans font-semibold text-[13px] text-foreground uppercase tracking-[0.08em]">
                                    {t("export.section.fields")}{" "}
                                    <span className="ml-1 font-mono text-[11px] text-muted-foreground tracking-normal">
                                        ({options.fieldIds.length}/{schema.fields.length})
                                    </span>
                                </h3>
                                <div className="inline-flex items-center gap-1 font-medium font-sans text-[11.5px] text-muted-foreground leading-none">
                                    <button type="button" onClick={selectAllFields} className="cursor-pointer rounded px-1.5 py-1 hover:bg-accent hover:text-foreground">
                                        {t("export.fields.all")}
                                    </button>
                                    <span aria-hidden>·</span>
                                    <button type="button" onClick={selectDefaultFields} className="cursor-pointer rounded px-1.5 py-1 hover:bg-accent hover:text-foreground">
                                        {t("export.fields.default")}
                                    </button>
                                    <span aria-hidden>·</span>
                                    <button type="button" onClick={clearAllFields} className="cursor-pointer rounded px-1.5 py-1 hover:bg-accent hover:text-foreground">
                                        {t("export.fields.none")}
                                    </button>
                                </div>
                            </div>

                            <div className="flex flex-col gap-3.5 rounded-lg border border-border bg-card/50 p-3">
                                {fieldsByGroup.map(([group, fields]) => {
                                    const groupIds = fields.map((f) => f.id);
                                    const allSelected = groupIds.every((id) => enabledSet.has(id));
                                    const someSelected = groupIds.some((id) => enabledSet.has(id));
                                    const toggleGroup = () => {
                                        if (allSelected) {
                                            setOptions((prev) => ({ ...prev, fieldIds: prev.fieldIds.filter((id) => !groupIds.includes(id)) }));
                                        } else {
                                            setOptions((prev) => {
                                                const order = schema.fields.map((f) => f.id);
                                                const next = Array.from(new Set([...prev.fieldIds, ...groupIds]));
                                                next.sort((a, b) => order.indexOf(a) - order.indexOf(b));
                                                return { ...prev, fieldIds: next };
                                            });
                                        }
                                    };
                                    return (
                                        <div key={group} className="flex flex-col gap-1.5">
                                            <button type="button" onClick={toggleGroup} className="flex cursor-pointer items-center gap-2 font-medium font-sans text-[11px] text-muted-foreground uppercase leading-none tracking-widest hover:text-foreground">
                                                <Checkbox checked={allSelected} indeterminate={!allSelected && someSelected} onCheckedChange={toggleGroup} aria-label={t("export.fields.toggleGroup", { group })} />
                                                <span>{group}</span>
                                                <span className="font-mono text-[10px] tracking-normal">
                                                    {fields.filter((f) => enabledSet.has(f.id)).length}/{fields.length}
                                                </span>
                                            </button>
                                            <div className="grid grid-cols-2 gap-x-3 gap-y-1 pl-6">
                                                {fields.map((f) => (
                                                    // biome-ignore lint/a11y/noLabelWithoutControl: Checkbox renders the input
                                                    <label key={f.id} className="flex cursor-pointer items-center gap-2 rounded px-1.5 py-1 hover:bg-accent/50">
                                                        <Checkbox checked={enabledSet.has(f.id)} onCheckedChange={() => toggleField(f.id)} />
                                                        <span className="truncate font-sans text-[12.5px] text-foreground leading-none">{f.label}</span>
                                                    </label>
                                                ))}
                                            </div>
                                        </div>
                                    );
                                })}
                            </div>
                        </section>

                        <section className="mt-6 flex flex-col gap-3">
                            <h3 className="font-sans font-semibold text-[13px] text-foreground uppercase tracking-[0.08em]">{t("export.section.options")}</h3>
                            <div className="flex flex-col gap-2.5 rounded-lg border border-border bg-card/50 p-3.5">
                                {supportsPretty && (
                                    <OptionRow label={t("export.option.pretty.label")} description={t("export.option.pretty.desc")}>
                                        <Switch checked={options.pretty} onCheckedChange={(v) => setOption("pretty", v)} />
                                    </OptionRow>
                                )}

                                {supportsIndent && (
                                    <OptionRow label={t("export.option.indent.label")} description={t("export.option.indent.desc")}>
                                        <Select value={options.jsonIndent} onValueChange={(v) => setOption("jsonIndent", v as IExportOptions["jsonIndent"])}>
                                            <SelectTrigger size="sm" className="w-32">
                                                <SelectValue />
                                            </SelectTrigger>
                                            <SelectContent>
                                                <SelectItem value="0">{t("export.option.indent.none")}</SelectItem>
                                                <SelectItem value="2">{t("export.option.indent.two")}</SelectItem>
                                                <SelectItem value="4">{t("export.option.indent.four")}</SelectItem>
                                                <SelectItem value="tab">{t("export.option.indent.tab")}</SelectItem>
                                            </SelectContent>
                                        </Select>
                                    </OptionRow>
                                )}

                                {isFlatFormat && (
                                    <>
                                        {(options.format === "csv" || options.format === "tsv") && (
                                            <OptionRow label={t("export.option.delimiter.label")} description={t("export.option.delimiter.desc")}>
                                                <Input value={options.csvDelimiter} onChange={(e) => setOption("csvDelimiter", e.target.value || (options.format === "tsv" ? "\t" : ","))} maxLength={3} className="w-20" size="sm" />
                                            </OptionRow>
                                        )}
                                        {(options.format === "csv" || options.format === "tsv") && (
                                            <OptionRow label={t("export.option.header.label")} description={t("export.option.header.desc")}>
                                                <Switch checked={options.csvHeaders} onCheckedChange={(v) => setOption("csvHeaders", v)} />
                                            </OptionRow>
                                        )}
                                        {(options.format === "csv" || options.format === "tsv") && (
                                            <OptionRow label={t("export.option.bom.label")} description={t("export.option.bom.desc")}>
                                                <Switch checked={options.csvBom} onCheckedChange={(v) => setOption("csvBom", v)} />
                                            </OptionRow>
                                        )}
                                        {options.format === "markdown" && (
                                            <OptionRow label={t("export.option.rowIndex.label")} description={t("export.option.rowIndex.desc")}>
                                                <Switch checked={options.markdownRowIndex} onCheckedChange={(v) => setOption("markdownRowIndex", v)} />
                                            </OptionRow>
                                        )}
                                        <OptionRow label={t("export.option.arrays.label")} description={t("export.option.arrays.desc")}>
                                            <Select value={options.arrayMode} onValueChange={(v) => setOption("arrayMode", v as IExportOptions["arrayMode"])}>
                                                <SelectTrigger size="sm" className="w-44">
                                                    <SelectValue />
                                                </SelectTrigger>
                                                <SelectContent>
                                                    <SelectItem value="join">{t("export.option.arrays.join")}</SelectItem>
                                                    <SelectItem value="json">{t("export.option.arrays.json")}</SelectItem>
                                                </SelectContent>
                                            </Select>
                                        </OptionRow>
                                        {options.arrayMode === "join" && (
                                            <OptionRow label={t("export.option.arraySep.label")} description={t("export.option.arraySep.desc")}>
                                                <Input value={options.arraySeparator} onChange={(e) => setOption("arraySeparator", e.target.value)} maxLength={8} className="w-24" size="sm" />
                                            </OptionRow>
                                        )}
                                    </>
                                )}

                                {supportsLineEnding && (
                                    <OptionRow label={t("export.option.lineEnding.label")} description={t("export.option.lineEnding.desc")}>
                                        <Select value={options.lineEnding} onValueChange={(v) => setOption("lineEnding", v as IExportOptions["lineEnding"])}>
                                            <SelectTrigger size="sm" className="w-32">
                                                <SelectValue />
                                            </SelectTrigger>
                                            <SelectContent>
                                                <SelectItem value="lf">{t("export.option.lineEnding.lf")}</SelectItem>
                                                <SelectItem value="crlf">{t("export.option.lineEnding.crlf")}</SelectItem>
                                            </SelectContent>
                                        </Select>
                                    </OptionRow>
                                )}

                                {options.format === "xml" && (
                                    <>
                                        <OptionRow label={t("export.option.xmlRoot.label")} description={t("export.option.xmlRoot.desc")}>
                                            <Input value={options.xmlRoot} onChange={(e) => setOption("xmlRoot", e.target.value)} className="w-40" size="sm" />
                                        </OptionRow>
                                        <OptionRow label={t("export.option.xmlItem.label")} description={t("export.option.xmlItem.desc")}>
                                            <Input value={options.xmlItem} onChange={(e) => setOption("xmlItem", e.target.value)} className="w-40" size="sm" />
                                        </OptionRow>
                                        <OptionRow label={t("export.option.xmlAttrs.label")} description={t("export.option.xmlAttrs.desc")}>
                                            <Switch checked={options.xmlAttributes} onCheckedChange={(v) => setOption("xmlAttributes", v)} />
                                        </OptionRow>
                                    </>
                                )}
                            </div>
                        </section>

                        <section className="mt-6 flex flex-col gap-2">
                            <h3 className="font-sans font-semibold text-[13px] text-foreground uppercase tracking-[0.08em]">{t("export.section.filename")}</h3>
                            <div className="flex items-center gap-2">
                                <Input value={filename} onChange={(e) => setFilename(e.target.value)} placeholder={schema.pluralName} className="flex-1" size="sm" />
                                <span className="font-mono text-[12px] text-muted-foreground">.{result?.extension ?? options.format}</span>
                            </div>
                            <p className="font-sans text-[11px] text-muted-foreground leading-snug">{t("export.filename.hint")}</p>
                        </section>
                    </div>

                    <div className="hidden min-h-0 flex-col border-border bg-muted/30 lg:flex lg:border-l">
                        <div className="flex items-center justify-between gap-2 border-b px-5 py-3">
                            <div className="flex flex-col gap-0.5">
                                <span className="font-sans font-semibold text-[12px] text-foreground uppercase tracking-[0.08em]">{t("export.preview.title")}</span>
                                <span className="font-mono text-[11px] text-muted-foreground">
                                    {fmt.number(sourceRows.length)} {sourceRows.length === 1 ? schema.itemName : schema.pluralName} · {t("export.preview.fields", { count: options.fieldIds.length })} · ~{formatBytes(bytes)}
                                </span>
                            </div>
                            <Button type="button" size="sm" variant="outline" onClick={handleCopy} disabled={!result}>
                                {copied ? <CheckIcon /> : <CopyIcon />}
                                {copied ? t("export.copied") : t("export.copy")}
                            </Button>
                        </div>
                        <div className="min-h-0 flex-1 overflow-auto">
                            {options.fieldIds.length === 0 ? (
                                <div className="grid h-full place-items-center p-6">
                                    <p className="text-center font-sans text-[13px] text-muted-foreground">{t("export.preview.empty")}</p>
                                </div>
                            ) : (
                                <pre className="m-0 whitespace-pre-wrap break-all p-4 font-mono text-[11.5px] text-foreground leading-relaxed">{previewText}</pre>
                            )}
                        </div>
                        {sourceRows.length > PREVIEW_ROW_LIMIT && <div className="border-t bg-background/50 px-5 py-2 font-sans text-[11px] text-muted-foreground">{t("export.preview.truncated", { limit: PREVIEW_ROW_LIMIT, total: fmt.number(sourceRows.length) })}</div>}
                    </div>
                </div>

                <DialogFooter>
                    <DialogClose render={<Button variant="outline" />}>{t("export.cancel")}</DialogClose>
                    <Button type="button" onClick={handleDownload} disabled={!result || sourceRows.length === 0}>
                        <DownloadIcon />
                        {t("export.download")} {result ? `(${formatBytes(bytes)})` : ""}
                    </Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    );
}

function OptionRow({ label, description, children }: { label: string; description?: string; children: React.ReactNode }): React.ReactElement {
    return (
        <div className="flex items-center justify-between gap-3">
            <div className="flex flex-col gap-0.5">
                <span className="font-medium font-sans text-[13px] text-foreground leading-none">{label}</span>
                {description && <span className="font-sans text-[11px] text-muted-foreground leading-snug">{description}</span>}
            </div>
            {children}
        </div>
    );
}

function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}
