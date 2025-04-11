export interface CdnOptions {
    cacheControl?: string;
    headers?: Record<string, string>;
    allowedExtensions?: string[];
}
