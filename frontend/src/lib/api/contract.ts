import { z } from "zod";

function report(name: string, issues: string[]): void {
    const message = `API contract mismatch in "${name}": ${issues.join("; ")}`;
    if (import.meta.env?.DEV) throw new Error(message);
    console.error(message);
}

export function checkSample<T>(name: string, schema: z.ZodType<T>, sample: unknown): void {
    if (sample === undefined || sample === null) return;
    const result = schema.safeParse(sample);
    if (result.success) return;
    report(
        name,
        result.error.issues.map((i) => `${i.path.join(".") || "(root)"}: ${i.message}`),
    );
}

export function firstValue(record: unknown): unknown {
    if (typeof record !== "object" || record === null) return undefined;
    const values = Object.values(record as Record<string, unknown>);
    return values.length > 0 ? values[0] : undefined;
}

export const voiceDataContract = z
    .object({
        voiceUrl: z.string().nullable(),
        language: z.string().nullable(),
        cvName: z.array(z.string()).nullable(),
    })
    .loose();

export const voiceContract = z
    .object({
        charWordId: z.string(),
        charId: z.string(),
        voiceAsset: z.string(),
        placeType: z.string(),
        data: z.array(voiceDataContract).nullish(),
    })
    .loose();

export const materialItemContract = z
    .object({
        itemId: z.string(),
        name: z.string(),
        iconId: z.string(),
        rarity: z.string(),
        classifyType: z.string(),
        itemType: z.string(),
        sortId: z.number(),
    })
    .loose();
