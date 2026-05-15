import { useEffect, useId, useState } from "react";
import { Field, FieldLabel } from "#/components/ui/field";
import { Input } from "#/components/ui/input";
import { cn } from "#/lib/utils";
import styles from "./Editor.module.css";

const PRESET_SWATCHES = ["#dc4d56", "#e0834a", "#d8b54a", "#86c057", "#5dbf86", "#52b9b3", "#5aa9d9", "#6f78d5", "#9b73d4", "#c069b4", "#e07a9b", "#8a8a8a"];

const HEX_RE = /^#[0-9a-fA-F]{6}$/;

interface IColorPickerProps {
    value: string;
    onChange: (next: string) => void;
}

export function ColorPicker({ value, onChange }: IColorPickerProps) {
    const [hex, setHex] = useState(value);
    const hexId = useId();

    useEffect(() => {
        setHex(value);
    }, [value]);

    const commitHex = (next: string) => {
        const trimmed = next.trim();
        const withHash = trimmed.startsWith("#") ? trimmed : `#${trimmed}`;
        if (HEX_RE.test(withHash)) onChange(withHash.toLowerCase());
    };

    const handleHexBlur = () => {
        if (!HEX_RE.test(hex)) setHex(value);
    };

    return (
        <div className="flex flex-col gap-3.5">
            <div>
                <p className="m-0 mb-2 font-bold font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.14em]">Presets</p>
                <div className="grid grid-cols-6 gap-1.5">
                    {PRESET_SWATCHES.map((c) => (
                        <button key={c} type="button" className={styles.swatch} style={{ background: c }} data-selected={c.toLowerCase() === value.toLowerCase() || undefined} aria-label={`Use color ${c}`} onClick={() => onChange(c)} />
                    ))}
                </div>
            </div>

            <Field>
                <FieldLabel htmlFor={hexId} className="text-xs">
                    Custom
                    <span className="ml-auto inline-flex items-center gap-1.5 font-medium font-mono text-[10.5px] text-muted-foreground">
                        <span className="inline-block h-3.5 w-3.5 rounded border border-border" style={{ background: HEX_RE.test(hex) ? hex : value }} aria-hidden="true" />
                        {value.toUpperCase()}
                    </span>
                </FieldLabel>
                <div className="flex w-full items-center gap-2">
                    <label
                        className={cn("relative inline-flex h-8 w-9 shrink-0 cursor-pointer items-center justify-center overflow-hidden rounded-lg border border-input", "focus-within:ring-2 focus-within:ring-ring focus-within:ring-offset-1 focus-within:ring-offset-background")}
                        style={{ background: HEX_RE.test(hex) ? hex : value }}
                    >
                        <input
                            type="color"
                            value={HEX_RE.test(hex) ? hex : value}
                            onChange={(e) => {
                                setHex(e.target.value);
                                onChange(e.target.value);
                            }}
                            className="absolute inset-0 cursor-pointer opacity-0"
                            aria-label="Open native color picker"
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
                        aria-invalid={!HEX_RE.test(hex) || undefined}
                        size="sm"
                    />
                </div>
            </Field>
        </div>
    );
}
