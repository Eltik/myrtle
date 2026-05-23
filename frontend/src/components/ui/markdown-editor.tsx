"use client";

import { BoldIcon, CodeIcon, EyeIcon, ItalicIcon, LinkIcon, ListIcon, ListOrderedIcon, PencilLineIcon, QuoteIcon } from "lucide-react";
import type React from "react";
import { useCallback, useId, useRef, useState } from "react";
import { Tabs, TabsList, TabsPanel, TabsTab } from "#/components/ui/tabs";
import { Markdown } from "#/lib/markdown";
import { cn } from "#/lib/utils";

export interface IMarkdownEditorProps {
    id?: string;
    name?: string;
    value: string;
    onChange: (next: string) => void;
    placeholder?: string;
    maxLength?: number;
    rows?: number;
    disabled?: boolean;
    autoFocus?: boolean;
    className?: string;
    textareaClassName?: string;
    /** Hide the formatting toolbar. */
    hideToolbar?: boolean;
    /** Visual density of the textarea. */
    size?: "sm" | "default";
    /** Inline aria invalid attribute, mirrors the Textarea API. */
    "aria-invalid"?: boolean;
    /** Helper hint shown under the editor (Markdown supported / shortcut text). */
    showHint?: boolean;
}

type ToolbarAction = "bold" | "italic" | "code" | "link" | "ul" | "ol" | "quote";

export function MarkdownEditor({ id, name, value, onChange, placeholder, maxLength, rows = 5, disabled = false, autoFocus = false, className, textareaClassName, hideToolbar = false, size = "default", "aria-invalid": ariaInvalid, showHint = true }: IMarkdownEditorProps): React.ReactElement {
    const fallbackId = useId();
    const inputId = id ?? fallbackId;
    const ref = useRef<HTMLTextAreaElement>(null);
    const [tab, setTab] = useState<"write" | "preview">("write");

    const apply = useCallback(
        (action: ToolbarAction) => {
            const ta = ref.current;
            if (!ta || disabled) return;
            const next = transform(ta, action, value, maxLength);
            if (next == null) return;
            onChange(next.value);
            // Restore selection after React re-renders.
            requestAnimationFrame(() => {
                const cur = ref.current;
                if (!cur) return;
                cur.focus();
                cur.setSelectionRange(next.selStart, next.selEnd);
            });
        },
        [disabled, maxLength, onChange, value],
    );

    const onKeyDown = useCallback(
        (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
            if (!(e.metaKey || e.ctrlKey)) return;
            const k = e.key.toLowerCase();
            if (k === "b") {
                e.preventDefault();
                apply("bold");
            } else if (k === "i") {
                e.preventDefault();
                apply("italic");
            } else if (k === "k") {
                e.preventDefault();
                apply("link");
            }
        },
        [apply],
    );

    const enforceMax = useCallback(
        (raw: string) => {
            return maxLength != null && raw.length > maxLength ? raw.slice(0, maxLength) : raw;
        },
        [maxLength],
    );

    const isWrite = tab === "write";

    return (
        <div
            className={cn(
                "flex w-full min-w-0 flex-col rounded-lg border border-input bg-background not-dark:bg-clip-padding shadow-xs/5 transition-shadow",
                "focus-within:border-ring focus-within:ring-[3px] focus-within:ring-ring/24",
                ariaInvalid && "border-destructive/36 focus-within:border-destructive/64 focus-within:ring-destructive/16",
                disabled && "opacity-64",
                "dark:bg-input/32",
                className,
            )}
            data-slot="markdown-editor"
            aria-disabled={disabled || undefined}
        >
            <Tabs value={tab} onValueChange={(v) => setTab(v as "write" | "preview")} className="gap-0">
                {/* Tabs row */}
                <div className="flex items-center gap-2 border-border/70 border-b px-1.5 py-1.5">
                    <TabsList variant="default" className="h-9 p-0.5 sm:h-7">
                        <TabsTab value="write" className="h-8 gap-1.5 px-2.5 text-[13px] sm:h-6 sm:px-2 sm:text-[12px]">
                            <PencilLineIcon className="size-4 sm:size-3.5" />
                            <span>Write</span>
                        </TabsTab>
                        <TabsTab value="preview" className="h-8 gap-1.5 px-2.5 text-[13px] sm:h-6 sm:px-2 sm:text-[12px]">
                            <EyeIcon className="size-4 sm:size-3.5" />
                            <span>Preview</span>
                        </TabsTab>
                    </TabsList>

                    {/* Desktop toolbar lives on this row */}
                    {!hideToolbar && isWrite && (
                        <div className="ml-auto hidden items-center gap-0.5 sm:flex" role="toolbar" aria-label="Formatting">
                            <ToolbarButtons apply={apply} disabled={disabled} />
                        </div>
                    )}
                </div>

                {/* Mobile toolbar gets its own scrollable row, only shown when on Write tab */}
                {!hideToolbar && isWrite && (
                    <div
                        className="scrollbar-thin -mb-px flex items-center gap-0.5 overflow-x-auto border-border/70 border-b px-1.5 py-1 sm:hidden"
                        role="toolbar"
                        aria-label="Formatting"
                        // Allow horizontal scroll without page bounce
                        style={{ scrollbarWidth: "thin", WebkitOverflowScrolling: "touch" }}
                    >
                        <ToolbarButtons apply={apply} disabled={disabled} mobile />
                    </div>
                )}

                <TabsPanel value="write" className="flex flex-col">
                    <textarea
                        ref={ref}
                        id={inputId}
                        name={name}
                        value={value}
                        onChange={(e) => onChange(enforceMax(e.target.value))}
                        onKeyDown={onKeyDown}
                        placeholder={placeholder}
                        rows={rows}
                        disabled={disabled}
                        // biome-ignore lint/a11y/noAutofocus: opt-in prop; mirrors native Textarea API used by dialogs that need first-focus
                        autoFocus={autoFocus}
                        aria-invalid={ariaInvalid || undefined}
                        spellCheck
                        className={cn(
                            // Base: 16px on mobile prevents iOS Safari auto-zoom on focus
                            "w-full resize-y bg-transparent px-3 py-2.5 font-sans text-[16px] text-foreground outline-none sm:text-sm",
                            "field-sizing-content min-h-32 sm:min-h-24",
                            size === "sm" && "px-2.5 py-2 sm:text-[13px]",
                            "placeholder:text-muted-foreground/70",
                            textareaClassName,
                        )}
                        data-slot="markdown-editor-input"
                    />
                </TabsPanel>

                <TabsPanel value="preview" className="flex flex-col">
                    <div className={cn("min-h-32 px-3 py-2.5 font-sans text-[15px] text-foreground sm:min-h-24 sm:text-sm", size === "sm" && "px-2.5 py-2 sm:text-[13px]")} style={{ minHeight: `${Math.max(rows, 2) * 1.6}em` }}>
                        {value.trim() ? <Markdown text={value} flush /> : <p className="m-0 font-sans text-muted-foreground/70 text-sm italic">Nothing to preview yet.</p>}
                    </div>
                </TabsPanel>
            </Tabs>

            {showHint && (
                <div className="flex flex-wrap items-center justify-between gap-x-2 gap-y-1 border-border/60 border-t px-2.5 py-1 font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.08em]">
                    <span className="hidden sm:inline">
                        <span className="font-semibold text-foreground/80">**bold**</span> · <span className="font-semibold text-foreground/80">*italic*</span> · <span className="font-semibold text-foreground/80">`code`</span> · <span className="font-semibold text-foreground/80">[link](url)</span>
                    </span>
                    <span className="sm:hidden">Markdown</span>
                    {maxLength != null && (
                        <span className="ml-auto tabular-nums">
                            <span className="text-foreground">{value.length}</span> / {maxLength}
                        </span>
                    )}
                </div>
            )}
        </div>
    );
}

function ToolbarButtons({ apply, disabled, mobile = false }: { apply: (a: ToolbarAction) => void; disabled?: boolean; mobile?: boolean }) {
    return (
        <>
            <FormatButton label="Bold (⌘B)" onClick={() => apply("bold")} disabled={disabled} mobile={mobile}>
                <BoldIcon className="size-4 sm:size-3.5" />
            </FormatButton>
            <FormatButton label="Italic (⌘I)" onClick={() => apply("italic")} disabled={disabled} mobile={mobile}>
                <ItalicIcon className="size-4 sm:size-3.5" />
            </FormatButton>
            <FormatButton label="Inline code" onClick={() => apply("code")} disabled={disabled} mobile={mobile}>
                <CodeIcon className="size-4 sm:size-3.5" />
            </FormatButton>
            <FormatButton label="Link (⌘K)" onClick={() => apply("link")} disabled={disabled} mobile={mobile}>
                <LinkIcon className="size-4 sm:size-3.5" />
            </FormatButton>
            <span className={cn("mx-1 h-5 w-px shrink-0 bg-border sm:h-4")} aria-hidden="true" />
            <FormatButton label="Bullet list" onClick={() => apply("ul")} disabled={disabled} mobile={mobile}>
                <ListIcon className="size-4 sm:size-3.5" />
            </FormatButton>
            <FormatButton label="Numbered list" onClick={() => apply("ol")} disabled={disabled} mobile={mobile}>
                <ListOrderedIcon className="size-4 sm:size-3.5" />
            </FormatButton>
            <FormatButton label="Quote" onClick={() => apply("quote")} disabled={disabled} mobile={mobile}>
                <QuoteIcon className="size-4 sm:size-3.5" />
            </FormatButton>
        </>
    );
}

function FormatButton({ label, onClick, disabled, mobile = false, children }: { label: string; onClick: () => void; disabled?: boolean; mobile?: boolean; children: React.ReactNode }) {
    return (
        <button
            type="button"
            // onMouseDown prevents the textarea from losing focus (and thus losing the selection) when the button is clicked.
            onMouseDown={(e) => e.preventDefault()}
            onClick={onClick}
            disabled={disabled}
            title={label}
            aria-label={label}
            className={cn(
                "relative inline-flex shrink-0 cursor-pointer items-center justify-center rounded-md text-muted-foreground transition-colors",
                "hover:bg-accent hover:text-foreground active:bg-accent",
                "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                "disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:bg-transparent",
                // 44pt tap target on touch devices (via invisible ::after extension)
                "touch-manipulation pointer-coarse:after:absolute pointer-coarse:after:size-full pointer-coarse:after:min-h-11 pointer-coarse:after:min-w-11",
                mobile ? "size-9" : "size-7",
            )}
        >
            {children}
        </button>
    );
}

interface ITransformResult {
    value: string;
    selStart: number;
    selEnd: number;
}

function transform(ta: HTMLTextAreaElement, action: ToolbarAction, value: string, maxLength: number | undefined): ITransformResult | null {
    const start = ta.selectionStart ?? 0;
    const end = ta.selectionEnd ?? 0;
    const before = value.slice(0, start);
    const middle = value.slice(start, end);
    const after = value.slice(end);

    let result: ITransformResult;

    switch (action) {
        case "bold":
            result = wrap(before, middle, after, "**", "**", "bold text");
            break;
        case "italic":
            result = wrap(before, middle, after, "*", "*", "italic text");
            break;
        case "code":
            result = wrap(before, middle, after, "`", "`", "code");
            break;
        case "link":
            result = insertLink(before, middle, after);
            break;
        case "ul":
            result = prefixLines(value, start, end, "- ");
            break;
        case "ol":
            result = prefixLines(value, start, end, "1. ", true);
            break;
        case "quote":
            result = prefixLines(value, start, end, "> ");
            break;
    }

    if (maxLength != null && result.value.length > maxLength) return null;
    return result;
}

function wrap(before: string, middle: string, after: string, opener: string, closer: string, placeholder: string): ITransformResult {
    if (middle) {
        // Toggle: if selection is already wrapped, unwrap it
        if (before.endsWith(opener) && after.startsWith(closer)) {
            const value = `${before.slice(0, -opener.length)}${middle}${after.slice(closer.length)}`;
            const selStart = before.length - opener.length;
            return { value, selStart, selEnd: selStart + middle.length };
        }
        const value = `${before}${opener}${middle}${closer}${after}`;
        const selStart = before.length + opener.length;
        return { value, selStart, selEnd: selStart + middle.length };
    }
    const value = `${before}${opener}${placeholder}${closer}${after}`;
    const selStart = before.length + opener.length;
    return { value, selStart, selEnd: selStart + placeholder.length };
}

function insertLink(before: string, middle: string, after: string): ITransformResult {
    if (middle) {
        const value = `${before}[${middle}](url)${after}`;
        const urlStart = before.length + middle.length + 3;
        return { value, selStart: urlStart, selEnd: urlStart + 3 };
    }
    const placeholder = "link text";
    const value = `${before}[${placeholder}](url)${after}`;
    const selStart = before.length + 1;
    return { value, selStart, selEnd: selStart + placeholder.length };
}

function prefixLines(value: string, start: number, end: number, prefix: string, ordered = false): ITransformResult {
    const lineStart = value.lastIndexOf("\n", start - 1) + 1;
    const lineEnd = end >= value.length ? value.length : value.indexOf("\n", end);
    const effectiveEnd = lineEnd === -1 ? value.length : lineEnd;
    const region = value.slice(lineStart, effectiveEnd);

    const lines = region.split("\n");
    const updated = lines
        .map((line, idx) => {
            if (!line && idx === 0 && lines.length === 1) return prefix;
            if (ordered) return `${idx + 1}. ${line.replace(/^(\s*)(\d+[.)]\s+|[-*+]\s+|>\s+)/, "$1")}`;
            return `${prefix}${line.replace(/^(\s*)(\d+[.)]\s+|[-*+]\s+|>\s+)/, "$1")}`;
        })
        .join("\n");

    const newValue = `${value.slice(0, lineStart)}${updated}${value.slice(effectiveEnd)}`;
    return {
        value: newValue,
        selStart: lineStart,
        selEnd: lineStart + updated.length,
    };
}
