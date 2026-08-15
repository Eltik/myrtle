import { DatabaseIcon, PaletteIcon } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle, SectionLabel, SettingRow, Switch } from "frontend";

const noop = () => {};

const HUES = ["#e5484d", "#f5a524", "#f2d024", "#6ec36e", "#3ba1c9", "#7c6cf0", "#c86cd0", "#8b8fa3"];

/** The Accent-colour block in /settings → Appearance: an icon-led label above a swatch row. */
export const WithIcon = () => (
    <Card>
        <CardHeader>
            <CardTitle>Accent color</CardTitle>
            <CardDescription>Tints buttons, links, focus rings, and the active state across the whole app.</CardDescription>
        </CardHeader>
        <CardContent className="pt-0">
            <SectionLabel icon={<PaletteIcon />}>Preset hue</SectionLabel>
            <div className="mb-5 grid max-w-110 grid-cols-6 gap-2.5 sm:grid-cols-12">
                {HUES.map((hex) => (
                    <span key={hex} className="relative aspect-square w-full rounded-full border-2 border-background ring-1 ring-border" style={{ backgroundColor: hex }} />
                ))}
            </div>
            <SectionLabel icon={<PaletteIcon />}>Custom color</SectionLabel>
            <span className="inline-flex items-center gap-2.5 rounded-lg border border-input bg-card px-2.5 py-1.5 font-medium font-sans text-[13px]">
                <span className="inline-block size-5 rounded-full border border-border bg-primary" />
                Pick custom hex
                <span className="font-mono text-[10px] text-muted-foreground/70 tabular-nums">#E5484D</span>
            </span>
        </CardContent>
    </Card>
);

/** Without an icon it is a plain uppercase divider between groups of rows. */
export const Plain = () => (
    <Card>
        <CardHeader>
            <CardTitle>Account &amp; data</CardTitle>
            <CardDescription>Everything we store is visible on your profile page.</CardDescription>
        </CardHeader>
        <CardContent className="pt-0">
            <SectionLabel>Stored on Myrtle</SectionLabel>
            <SettingRow layout="inline" title="Store gacha history" description="Saves your synced pulls so you can browse them in Gacha → History." control={<Switch checked={true} onCheckedChange={noop} />} />
            <div className="h-4" />
            <SectionLabel icon={<DatabaseIcon />}>Portability</SectionLabel>
            <SettingRow layout="inline" title="Request data export" description="Email privacy@myrtle.moe to receive a portable copy of your data." control={<span className="font-mono text-[12px] text-muted-foreground">GDPR · 30 days</span>} />
        </CardContent>
    </Card>
);
