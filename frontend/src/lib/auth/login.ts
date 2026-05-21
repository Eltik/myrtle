import { z } from "zod";

export const AKServerSchema = z.enum(["en", "jp", "kr", "cn", "bili", "tw"]);
export type AKServer = z.infer<typeof AKServerSchema>;

interface IServerMeta {
    code: AKServer;
    region: string;
    publisher: string;
    /** True when authentication for this server isn't supported yet. */
    loginDisabled: boolean;
}

export const SERVERS: readonly IServerMeta[] = [
    { code: "en", region: "Global", publisher: "Yostar", loginDisabled: false },
    { code: "jp", region: "Japan", publisher: "Yostar", loginDisabled: false },
    { code: "kr", region: "Korea", publisher: "Yostar", loginDisabled: false },
    { code: "cn", region: "China", publisher: "Hypergryph", loginDisabled: true },
    { code: "tw", region: "Taiwan", publisher: "Longcheng", loginDisabled: true },
    { code: "bili", region: "Bilibili", publisher: "bilibili", loginDisabled: true },
];

const SERVER_BY_CODE: Record<string, IServerMeta> = Object.fromEntries(SERVERS.map((s) => [s.code, s]));

/** "EN · Yostar Global" - used in the settings page. */
export function formatServerWithPublisher(code: string | null | undefined): string {
    if (!code) return "-";
    const meta = SERVER_BY_CODE[code.toLowerCase()];
    if (!meta) return code.toUpperCase();
    return `${meta.code.toUpperCase()} · ${meta.publisher} ${meta.region}`;
}

/** "Global (EN)" - used in the login server picker. */
export function formatServerForPicker(code: AKServer): string {
    const meta = SERVER_BY_CODE[code];
    if (!meta) return code.toUpperCase();
    if (meta.code === "bili") return "Bilibili";
    return `${meta.region} (${meta.code.toUpperCase()})`;
}

export const loginSchema = z.object({
    email: z.email("Invalid email format").min(1, "Email is required").max(254, "Email too long"),
    code: z.union([z.string(), z.number()]).transform((val) => {
        const str = typeof val === "number" ? String(val) : val.trim();
        if (!/^\d{1,6}$/.test(str)) throw new Error("Code must be a 6-digit number");
        return str.padStart(6, "0");
    }),
    server: AKServerSchema.default("en"),
});

export type LoginInput = z.infer<typeof loginSchema>;
