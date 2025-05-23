import path from "node:path";

export const env = {
    LOAD_AK_CONFIG: process.env.LOAD_AK_CONFIG?.toLowerCase() === "true",
    DATABASE_URL: process.env.DATABASE_URL ?? "postgresql://postgres:password@localhost:5432",
    DATABASE_NAME: process.env.DATABASE_NAME ?? "myrtle",
    PORT: isNaN(Number(process.env.PORT)) ? 3060 : Number(process.env.PORT),
    REDIS_URL: process.env.REDIS_URL,
    REDIS_CACHE_TIME: Number(process.env.REDIS_CACHE_TIME) ?? 60 * 60 * 24 * 7,
    GITHUB_TOKEN: process.env.GITHUB_TOKEN,
    UNPACKED_DIR: process.env.UNPACKED_DIR ?? (process.env.NODE_ENV === "development" || !process.env.NODE_ENV ? path.join(import.meta.dir, "../../assets/Unpacked") : path.join(import.meta.dir, "../assets/Unpacked")),
};
