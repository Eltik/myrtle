import { z } from "zod";

export const AKServerSchema = z.enum(["en", "jp", "kr", "cn", "bili", "tw"]);
export type AKServer = z.infer<typeof AKServerSchema>;

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
