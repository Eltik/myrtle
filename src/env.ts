export const env = {
    PORT: isNaN(Number(process.env.PORT)) ? 3000 : Number(process.env.PORT),
    REDIS_URL: process.env.REDIS_URL,
    REDIS_CACHE_TIME: Number(process.env.REDIS_CACHE_TIME) ?? 60 * 60 * 24 * 7,
};
