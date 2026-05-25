export type Role = "Vanguard" | "Supporter" | "Defender" | "Sniper" | "Caster" | "Medic" | "Guard" | "Specialist";

export type Accent = "coral" | "mint" | "amber" | "violet";

export const ROLE_GRADIENT: Record<Role, string> = {
    Vanguard: "linear-gradient(135deg, oklch(0.7 0.12 55), oklch(0.55 0.14 45))",
    Caster: "linear-gradient(135deg, oklch(0.65 0.18 290), oklch(0.5 0.2 285))",
    Medic: "linear-gradient(135deg, oklch(0.72 0.14 180), oklch(0.58 0.14 175))",
    Defender: "linear-gradient(135deg, oklch(0.7 0.1 85), oklch(0.58 0.12 75))",
    Supporter: "linear-gradient(135deg, oklch(0.68 0.14 330), oklch(0.55 0.16 320))",
    Sniper: "linear-gradient(135deg, oklch(0.72 0.14 155), oklch(0.55 0.14 150))",
    Guard: "linear-gradient(135deg, oklch(0.65 0.2 20), oklch(0.52 0.2 15))",
    Specialist: "linear-gradient(135deg, oklch(0.7 0.14 230), oklch(0.55 0.15 225))",
};

export const ROLE_CHIP_GRADIENT: Record<Role, string> = {
    ...ROLE_GRADIENT,
    Guard: "linear-gradient(135deg, oklch(0.72 0.13 25), oklch(0.55 0.15 20))",
    Specialist: "linear-gradient(135deg, oklch(0.68 0.13 230), oklch(0.52 0.14 220))",
};

export const ROLE_DRAWER_GRADIENT: Record<Role, string> = {
    Vanguard: "linear-gradient(135deg, oklch(0.7 0.12 55 / 0.3), oklch(0.55 0.14 45 / 0.3))",
    Caster: "linear-gradient(135deg, oklch(0.65 0.18 290 / 0.3), oklch(0.5 0.2 285 / 0.3))",
    Medic: "linear-gradient(135deg, oklch(0.72 0.14 180 / 0.3), oklch(0.58 0.14 175 / 0.3))",
    Defender: "linear-gradient(135deg, oklch(0.7 0.1 85 / 0.3), oklch(0.58 0.12 75 / 0.3))",
    Supporter: "linear-gradient(135deg, oklch(0.68 0.14 330 / 0.3), oklch(0.55 0.16 320 / 0.3))",
    Sniper: "linear-gradient(135deg, oklch(0.72 0.14 155 / 0.3), oklch(0.55 0.14 150 / 0.3))",
    Guard: "linear-gradient(135deg, oklch(0.65 0.2 20 / 0.3), oklch(0.52 0.2 15 / 0.3))",
    Specialist: "linear-gradient(135deg, oklch(0.7 0.14 230 / 0.3), oklch(0.55 0.15 225 / 0.3))",
};

export const ROLE_SOLID: Record<Role, string> = {
    Vanguard: "oklch(0.6 0.12 55)",
    Caster: "oklch(0.55 0.18 290)",
    Medic: "oklch(0.62 0.14 180)",
    Defender: "oklch(0.6 0.1 85)",
    Supporter: "oklch(0.58 0.14 330)",
    Sniper: "oklch(0.62 0.14 155)",
    Guard: "oklch(0.62 0.13 25)",
    Specialist: "oklch(0.58 0.13 230)",
};

export const TL_ACCENT: Record<Accent, { a: string; b: string }> = {
    coral: { a: "oklch(0.78 0.14 25)", b: "oklch(0.55 0.16 25)" },
    mint: { a: "oklch(0.78 0.14 160)", b: "oklch(0.5 0.14 160)" },
    amber: { a: "oklch(0.82 0.14 85)", b: "oklch(0.58 0.14 75)" },
    violet: { a: "oklch(0.72 0.16 290)", b: "oklch(0.5 0.18 290)" },
};

export const FEAT_ACCENT: Record<Accent, { strip: string; iconColor: string; iconBg: string; iconBorder: string }> = {
    coral: {
        strip: "oklch(0.75 0.15 25 / 0.8)",
        iconColor: "var(--primary)",
        iconBg: "color-mix(in srgb, var(--primary) 10%, transparent)",
        iconBorder: "color-mix(in srgb, var(--primary) 20%, transparent)",
    },
    mint: {
        strip: "oklch(0.75 0.14 165 / 0.8)",
        iconColor: "oklch(0.65 0.14 165)",
        iconBg: "oklch(0.65 0.14 165 / 0.1)",
        iconBorder: "oklch(0.65 0.14 165 / 0.2)",
    },
    amber: {
        strip: "oklch(0.78 0.13 80 / 0.8)",
        iconColor: "oklch(0.7 0.13 80)",
        iconBg: "oklch(0.7 0.13 80 / 0.1)",
        iconBorder: "oklch(0.7 0.13 80 / 0.2)",
    },
    violet: {
        strip: "oklch(0.72 0.16 290 / 0.8)",
        iconColor: "oklch(0.6 0.16 290)",
        iconBg: "oklch(0.6 0.16 290 / 0.1)",
        iconBorder: "oklch(0.6 0.16 290 / 0.2)",
    },
};

export function roleVars(role: string): React.CSSProperties {
    const r = role as Role;
    const grad = ROLE_GRADIENT[r];
    const solid = ROLE_SOLID[r];
    if (!grad) return {};
    return { ["--role-grad" as never]: grad, ["--role-solid" as never]: solid };
}

export function tlAccentVars(accent: string): React.CSSProperties {
    const a = accent as Accent;
    const v = TL_ACCENT[a];
    if (!v) return {};
    return { ["--tl-a" as never]: v.a, ["--tl-b" as never]: v.b };
}

export function featAccentVars(accent: string): React.CSSProperties {
    const a = accent as Accent;
    const v = FEAT_ACCENT[a];
    if (!v) return {};
    return {
        ["--feat-strip" as never]: v.strip,
        ["--feat-icon-color" as never]: v.iconColor,
        ["--feat-icon-bg" as never]: v.iconBg,
        ["--feat-icon-border" as never]: v.iconBorder,
    };
}
