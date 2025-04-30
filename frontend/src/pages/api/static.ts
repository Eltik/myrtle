import { type ServerResponse } from "http";
import type Module from "module";
import { unstable_cache } from "next/cache";
import { env } from "~/env.js";
import type { Item } from "~/types/impl/api/static/material";
import type { ModuleData, Modules } from "~/types/impl/api/static/modules";
import type { Operator } from "~/types/impl/api/static/operator";
import type { Ranges } from "~/types/impl/api/static/ranges";
import type { Skill } from "~/types/impl/api/static/skills";
import type { Skin, SkinData } from "~/types/impl/api/static/skins";
import type { Voice, Voices } from "~/types/impl/api/static/voices";

// Cache configuration
const CACHE_TAG = "static-api";
const CACHE_TTL = 3600; // Cache lifetime in seconds (1 hour)

// Determine if we're in development mode
const isDevelopment = env.NODE_ENV === "development";

// Function to fetch data from backend without caching
const fetchWithoutCache = async <T>(endpoint: string, body: object): Promise<T> => {
    const response = await fetch(`${env.BACKEND_URL}${endpoint}`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(body),
    });

    if (!response.ok) {
        throw new Error(`Backend request failed with status ${response.status}`);
    }

    return response.json() as Promise<T>;
};

// Function to fetch data from backend with caching
const fetchWithCache = unstable_cache(
    async <T>(endpoint: string, body: object, _: string[] = []): Promise<T> => {
        const response = await fetch(`${env.BACKEND_URL}${endpoint}`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(body),
        });

        if (!response.ok) {
            throw new Error(`Backend request failed with status ${response.status}`);
        }

        return response.json() as Promise<T>;
    },
    // Cache key generation function as string[]
    ["static-api-cache"],
    // Cache options
    {
        tags: [CACHE_TAG],
        revalidate: CACHE_TTL,
    },
);

// Function that decides whether to use cache or not based on environment
const fetchData = async <T>(endpoint: string, body: object, cacheKey?: string[]): Promise<T> => {
    // In development mode, always bypass cache
    if (isDevelopment) {
        console.log("Development mode: Cache disabled");
        return fetchWithoutCache<T>(endpoint, body);
    }

    // In production, use cache if a cache key is provided
    if (cacheKey) {
        return fetchWithCache<T>(endpoint, body, cacheKey);
    }

    // Default fallback to non-cached fetch
    return fetchWithoutCache<T>(endpoint, body);
};

export default async function handler(request: Request, response: ServerResponse) {
    try {
        switch (request.body.type) {
            case "materials": {
                const requestBody = {
                    type: request.body.type,
                    id: request.body.id,
                };

                const materials = await fetchData<{ materials: Item[] }>("/static", requestBody, isDevelopment ? undefined : [`${CACHE_TAG}-materials-${request.body.id ?? "all"}`]);

                response.writeHead(200, { "Content-Type": "application/json" });
                response.write(
                    JSON.stringify({
                        data: materials.materials,
                    }),
                );
                return response.end();
            }
            case "modules": {
                const requestBody = {
                    type: request.body.type,
                    id: request.body.id,
                    method: request.body.method,
                };

                const modules = await fetchData<{ modules: Modules } | { details: ModuleData } | { modules: Module[] } | { modules: Module }>("/static", requestBody, isDevelopment ? undefined : [`${CACHE_TAG}-modules-${request.body.id ?? "all"}-${request.body.method ?? "default"}`]);

                response.writeHead(200, { "Content-Type": "application/json" });
                response.write(JSON.stringify(modules));
                return response.end();
            }
            case "operators": {
                const requestBody = {
                    type: request.body.type,
                    id: request.body.id,
                    fields: request.body.fields,
                };

                // Generate a cache key based on ID and sorted fields
                const fieldsKey = request.body.fields ? [...request.body.fields].sort().join(",") : "all";
                const cacheKey = isDevelopment ? undefined : [`${CACHE_TAG}-operators-${request.body.id ?? "all"}-fields-${fieldsKey}`];

                // Use fetchData for caching and allow partial operator types
                const operators = await fetchData<{ operators: Partial<Operator>[] | Partial<Operator> }>("/static", requestBody, cacheKey);

                response.writeHead(200, { "Content-Type": "application/json" });
                response.write(
                    JSON.stringify({
                        data: operators.operators,
                    }),
                );
                return response.end();
            }
            case "ranges": {
                const requestBody = {
                    type: request.body.type,
                    id: request.body.id,
                };

                const ranges = await fetchData<{ range: Ranges }>("/static", requestBody, isDevelopment ? undefined : [`${CACHE_TAG}-ranges-${request.body.id ?? "all"}`]);

                response.writeHead(200, { "Content-Type": "application/json" });
                response.write(
                    JSON.stringify({
                        data: ranges.range,
                    }),
                );
                return response.end();
            }
            case "skills": {
                const requestBody = {
                    type: request.body.type,
                    id: request.body.id,
                };

                const skills = await fetchData<{ skills: Skill[] }>("/static", requestBody, isDevelopment ? undefined : [`${CACHE_TAG}-skills-${request.body.id ?? "all"}`]);

                response.writeHead(200, { "Content-Type": "application/json" });
                response.write(
                    JSON.stringify({
                        data: skills.skills,
                    }),
                );
                return response.end();
            }
            case "trust": {
                const requestBody = {
                    type: request.body.type,
                    id: request.body.id,
                    trust: request.body.trust,
                };

                // Trust data can be modified, so we need to be careful with caching
                // Only cache GET requests (without trust parameter) and not in development
                const useCache = request.body.trust === undefined && !isDevelopment;

                let trust;
                if (useCache) {
                    trust = await fetchWithCache<{ trust: number | null }>("/static", requestBody, [`${CACHE_TAG}-trust-${request.body.id ?? "all"}`]);
                } else {
                    // For updates or in development, bypass cache
                    trust = await fetchWithoutCache<{ trust: number | null }>("/static", requestBody);
                }

                response.writeHead(200, { "Content-Type": "application/json" });
                response.write(
                    JSON.stringify({
                        data: trust.trust,
                    }),
                );
                return response.end();
            }
            case "handbook": {
                const requestBody = {
                    type: request.body.type,
                    id: request.body.id,
                };

                const handbook = await fetchData<{ handbook: string }>("/static", requestBody, isDevelopment ? undefined : [`${CACHE_TAG}-handbook-${request.body.id ?? "all"}`]);

                response.writeHead(200, { "Content-Type": "application/json" });
                response.write(
                    JSON.stringify({
                        data: handbook.handbook,
                    }),
                );
                return response.end();
            }
            case "skins": {
                const requestBody = {
                    type: request.body.type,
                    id: request.body.id,
                };

                const skins = await fetchData<{ skins: Skin[] | SkinData }>("/static", requestBody, isDevelopment ? undefined : [`${CACHE_TAG}-skins-${request.body.id ?? "all"}`]);

                response.writeHead(200, { "Content-Type": "application/json" });
                response.write(JSON.stringify(skins));
                return response.end();
            }
            case "voices": {
                const requestBody = {
                    type: request.body.type,
                    id: request.body.id,
                };

                //const voices = await fetchWithCache<{ voices: Voice[] | Voices }>("/static", requestBody, [`${CACHE_TAG}-voices-${request.body.id ?? "all"}`]);
                const voices = await fetchWithoutCache<{ voices: Voice[] | Voices }>("/static", requestBody);
                response.writeHead(200, { "Content-Type": "application/json" });
                response.write(JSON.stringify(voices));
                return response.end();
            }
            default:
                response.writeHead(400, { "Content-Type": "application/json" });
                response.write(
                    JSON.stringify({
                        error: "Invalid type.",
                    }),
                );
                return response.end();
        }
    } catch (error) {
        console.error("API error:", error);
        response.writeHead(500, { "Content-Type": "application/json" });
        response.write(
            JSON.stringify({
                error: "An error occurred while processing the request.",
            }),
        );
        return response.end();
    }
}

interface Request {
    body: {
        type: "materials" | "modules" | "operators" | "ranges" | "skills" | "trust" | "handbook" | "skins" | "voices";
        id?: string;
        method?: string;
        trust?: number;
        fields?: string[];
    };
}
