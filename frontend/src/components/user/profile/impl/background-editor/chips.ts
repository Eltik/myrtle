/**
 * A count after a label: smaller, monospaced and quiet, so the label reads first. `opacity-55`
 * measured 2.57 light / 2.58 dark (audit-003 U16). Subtle passes off a tint (5.29 / 5.51 on the
 * background) but not on a pressed primary tint (4.18 light on /15), so inside a pressed or
 * chosen row it steps up to muted-foreground (5.64 / 5.10 on /15, 4.97 dark on /12 over card).
 */
export const COUNT = "font-mono text-[10.5px] tabular-nums text-subtle-foreground in-aria-pressed:text-muted-foreground in-data-pressed:text-muted-foreground";

/** A toggle-group item whose pressed state reads at a glance: tinted with the primary, not the input grey. */
export const SEGMENT_ITEM = "data-pressed:border-primary/50 data-pressed:bg-primary/15 data-pressed:text-foreground dark:data-pressed:bg-primary/15";
