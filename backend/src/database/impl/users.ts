import { z } from "zod";
import type { User } from "../../types/impl/lib/impl/user/impl/get";
import { withDbMetadata } from "..";
import { env } from "../../env";

export const tableName = `${env.DATABASE_NAME}_users`;

export const userSchema = z.object({
    id: withDbMetadata(z.string(), { primaryKey: true, notNull: true, defaultValue: "gen_random_uuid()" }),
    uid: withDbMetadata(z.string(), { notNull: true }),
    server: withDbMetadata(z.string(), { notNull: true }),
    data: withDbMetadata(z.custom<User>(), { notNull: true }),
    created_at: withDbMetadata(z.string(), { defaultValue: "now()" }),
});
