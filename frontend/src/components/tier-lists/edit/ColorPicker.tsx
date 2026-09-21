import { useEffect, useId, useState } from "react";
import { Field, FieldLabel } from "#/components/ui/field";
import { Input } from "#/components/ui/input";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { isHexColor, normalizeHexColor } from "../shared";
import type { messages } from "./ColorPicker.messages";
import styles from "./Editor.module.css";

const PRESET_SWATCHES = ["#dc4d56", "#e0834a", "#d8b54a", "#86c057", "#5dbf86", "#52b9b3", "#5aa9d9", "#6f78d5", "#9b73d4", "#c069b4", "#e07a9b", "#8a8a8a"];

interface IColorPickerProps {
    value: string;
    onChange: (next: string) => void;
}

export function ColorPicker({ value, onChange }: IColorPickerProps) {
    const t: TypedT<typeof messages> = useT("tierLists");
    const [hex, setHex] = useState(value);
    const hexId = useId();

    useEffect(() => {
        setHex(value);
    }, [value]);

    // The swatch and native picker follow the typed hex as soon as it parses, else the committed value.
    const previewColor = isHexColor(hex) ? hex : value;

    const commitHex = (next: string) => {
        const normalized = normalizeHexColor(next);
        if (normalized) onChange(normalized);
    };

    const handleHexBlur = () => {
        if (!isHexColor(hex)) setHex(value);
    };

    return (
        <div className="flex flex-col gap-3.5">
            <div>
                <p className="m-0 mb-2 font-bold font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.14em]">{t("edit.color.presets")}</p>
                <div className="grid grid-cols-6 gap-1.5">
                    {PRESET_SWATCHES.map((c) => (
                        <button key={c} type="button" className={styles.swatch} style={{ background: c }} data-selected={c.toLowerCase() === value.toLowerCase() || undefined} aria-label={t("edit.color.useColor", { hex: c })} onClick={() => onChange(c)} />
                    ))}
                </div>
            </div>

            <Field>
                <FieldLabel htmlFor={hexId} className="text-xs">
                    {t("edit.color.custom")}
                    <span className="ml-auto inline-flex items-center gap-1.5 font-medium font-mono text-[10.5px] text-muted-foreground">
                        <span className="inline-block h-3.5 w-3.5 rounded border border-border" style={{ background: previewColor }} aria-hidden="true" />
                        {value.toUpperCase()}
                    </span>
                </FieldLabel>
                <div className="flex w-full items-center gap-2">
                    <label className={cn("relative inline-flex h-8 w-9 shrink-0 cursor-pointer items-center justify-center overflow-hidden rounded-lg border border-input", "focus-within:ring-2 focus-within:ring-ring focus-within:ring-offset-1 focus-within:ring-offset-background")} style={{ background: previewColor }}>
                        <input
                            type="color"
                            value={previewColor}
                            onChange={(e) => {
                                setHex(e.target.value);
                                onChange(e.target.value);
                            }}
                            className="absolute inset-0 cursor-pointer opacity-0"
                            aria-label={t("edit.color.nativePicker")}
                        />
                    </label>
                    <Input
                        id={hexId}
                        value={hex}
                        onChange={(e) => {
                            const next = (e.target as HTMLInputElement).value;
                            setHex(next);
                            commitHex(next);
                        }}
                        onBlur={handleHexBlur}
                        placeholder="#dc4d56"
                        aria-invalid={!isHexColor(hex) || undefined}
                        size="sm"
                    />
                </div>
            </Field>
        </div>
    );
}
