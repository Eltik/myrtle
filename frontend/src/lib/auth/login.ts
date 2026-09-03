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
    // Disabled on the frontend for now: the oauth2 grant this depends on uses
    // a placeholder appCode (Skland's, not the real game client's - see
    // backend core::hypergryph::passport module docs), so login fails even
    // with correct credentials. Backend implementation is left as-is.
    { code: "cn", region: "China", publisher: "Hypergryph", loginDisabled: true },
    { code: "tw", region: "Taiwan", publisher: "Longcheng", loginDisabled: true },
    // Disabled on the frontend for now: password login needs re-verification
    // against a real account (see the RSA cipher-key fix in
    // core::hypergryph::bilibili), and SMS login's endpoints are an
    // unconfirmed guess that 404s. Backend implementation is left as-is.
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

export const bilibiliLoginSchema = z.object({
    username: z.string().min(1, "Username is required").max(254, "Username too long"),
    password: z.string().min(1, "Password is required").max(254, "Password too long"),
});
export type BilibiliLoginInput = z.infer<typeof bilibiliLoginSchema>;

// Bilibili SMS-code login: phone plus the code requested for it, as an
// alternative to the username/password flow above. UNVERIFIED: see backend
// core::hypergryph::bilibili module docs - the SMS endpoints are a
// structural guess mirroring the confirmed password flow, not confirmed
// against a real response.
export const bilibiliSmsLoginSchema = z.object({
    phone: z.string().min(1, "Phone number is required").max(32, "Phone number too long"),
    code: z.string().min(1, "Code is required").max(16, "Code too long"),
});
export type BilibiliSmsLoginInput = z.infer<typeof bilibiliSmsLoginSchema>;

// Phone + either a password or an SMS code. Format is deliberately loose (no
// digit-count enforcement like the Yostar OTP above): the CN passport flow is
// experimental and its exact SMS code length hasn't been confirmed against a
// real account yet.
export const cnLoginSchema = z
    .object({
        phone: z.string().min(1, "Phone number is required").max(32, "Phone number too long"),
        password: z.string().min(1).max(254).optional(),
        code: z.string().min(1).max(16).optional(),
    })
    .refine((data) => Boolean(data.password) || Boolean(data.code), {
        message: "Provide either a password or a code",
    });
export type CnLoginInput = z.infer<typeof cnLoginSchema>;
