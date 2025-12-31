"use client";

import { Monitor, Moon, Palette, RotateCcw, Sun } from "lucide-react";
import { useTheme } from "next-themes";
import { useEffect, useState } from "react";
import { HueSlider } from "~/components/ui/hue-slider";
import { Button } from "~/components/ui/shadcn/button";
import { DropdownMenu, DropdownMenuContent, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger } from "~/components/ui/shadcn/dropdown-menu";
import { useAccentColorSafe } from "~/context/accent-color-context";
import { COLOR_PRESETS } from "~/lib/color-utils";
import { cn } from "~/lib/utils";

export function ThemeToggle() {
    const { theme, setTheme } = useTheme();
    const { hue, isDefault, setHue, resetToDefault } = useAccentColorSafe();
    const [mounted, setMounted] = useState(false);

    useEffect(() => {
        setMounted(true);
    }, []);

    if (!mounted) {
        return (
            <Button className="h-8 w-8" size="icon" variant="ghost">
                <Sun className="h-4 w-4" />
                <span className="sr-only">Toggle theme</span>
            </Button>
        );
    }

    return (
        <DropdownMenu>
            <DropdownMenuTrigger asChild>
                <Button className="h-8 w-8" size="icon" variant="ghost">
                    <Sun className="dark:-rotate-90 h-4 w-4 rotate-0 scale-100 transition-all dark:scale-0" />
                    <Moon className="absolute h-4 w-4 rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100" />
                    <span className="sr-only">Toggle theme</span>
                </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-64">
                {/* Theme Mode Section */}
                <DropdownMenuLabel className="flex items-center gap-2 font-medium text-muted-foreground text-xs">
                    <Sun className="h-3.5 w-3.5" />
                    Appearance
                </DropdownMenuLabel>
                <div className="grid grid-cols-3 gap-1 p-1">
                    <Button className={cn("h-8 justify-center gap-1.5 text-xs", theme === "light" && "bg-accent")} onClick={() => setTheme("light")} size="sm" variant="ghost">
                        <Sun className="h-3.5 w-3.5" />
                        Light
                    </Button>
                    <Button className={cn("h-8 justify-center gap-1.5 text-xs", theme === "dark" && "bg-accent")} onClick={() => setTheme("dark")} size="sm" variant="ghost">
                        <Moon className="h-3.5 w-3.5" />
                        Dark
                    </Button>
                    <Button className={cn("h-8 justify-center gap-1.5 text-xs", theme === "system" && "bg-accent")} onClick={() => setTheme("system")} size="sm" variant="ghost">
                        <Monitor className="h-3.5 w-3.5" />
                        Auto
                    </Button>
                </div>

                <DropdownMenuSeparator />

                {/* Accent Color Section */}
                <DropdownMenuLabel className="flex items-center justify-between font-medium text-muted-foreground text-xs">
                    <span className="flex items-center gap-2">
                        <Palette className="h-3.5 w-3.5" />
                        Accent Color
                    </span>
                    {!isDefault && (
                        <Button className="h-4 gap-1 px-1.5 text-[10px]" onClick={resetToDefault} size="sm" variant="ghost">
                            <RotateCcw className="h-3 w-3" />
                            Reset
                        </Button>
                    )}
                </DropdownMenuLabel>

                {/* Preset Colors */}
                <div className="flex flex-wrap gap-1 px-2 py-1.5">
                    {COLOR_PRESETS.map((preset) => (
                        <button
                            aria-label={`Set accent color to ${preset.name}`}
                            className={cn("h-5 w-5 rounded-full border-2 transition-transform hover:scale-110 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2", Math.abs(hue - preset.hue) < 5 ? "scale-110 border-foreground" : "border-transparent")}
                            key={preset.name}
                            onClick={() => setHue(preset.hue)}
                            style={{ backgroundColor: preset.color }}
                            title={preset.name}
                            type="button"
                        />
                    ))}
                </div>

                {/* Hue Slider */}
                <div className="px-2 pt-1 pb-2">
                    <HueSlider onChange={setHue} value={hue} />
                </div>
            </DropdownMenuContent>
        </DropdownMenu>
    );
}
