import { type ServerResponse } from "http";
import type Module from "module";
import { unstable_cache } from "next/cache";
import { env } from "~/env.js";
import type { Item } from "~/types/impl/api/static/material";
import type { ModuleData, Modules } from "~/types/impl/api/static/modules";
import type { Operator } from "~/types/impl/api/static/operator";
import type { Ranges } from "~/types/impl/api/static/ranges";
import type { Skill } from "~/types/impl/api/static/skills";

// Cache configuration
const CACHE_TAG = "static-api";
const CACHE_TTL = 3600; // Cache lifetime in seconds (1 hour)

// Function to fetch data from backend with caching
const fetchWithCache = unstable_cache(
    async <T>(endpoint: string, body: object, tags: string[] = []): Promise<T> => {
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

export default async function handler(request: Request, response: ServerResponse) {
    try {
        switch (request.body.type) {
            case "materials": {
                const requestBody = {
                    type: request.body.type,
                    id: request.body.id,
                };

                const materials = await fetchWithCache<{ materials: Item[] }>("/static", requestBody, [`${CACHE_TAG}-materials-${request.body.id ?? "all"}`]);

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

                const modules = await fetchWithCache<{ modules: Modules } | { details: ModuleData } | { modules: Module[] } | { modules: Module }>("/static", requestBody, [`${CACHE_TAG}-modules-${request.body.id ?? "all"}-${request.body.method ?? "default"}`]);

                response.writeHead(200, { "Content-Type": "application/json" });
                response.write(JSON.stringify(modules));
                return response.end();
            }
            case "operators": {
                const requestBody = {
                    type: request.body.type,
                    id: request.body.id,
                };

                const operators = await fetchWithCache<{ operators: Operator[] }>("/static", requestBody, [`${CACHE_TAG}-operators-${request.body.id ?? "all"}`]);

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

                const ranges = await fetchWithCache<{ range: Ranges }>("/static", requestBody, [`${CACHE_TAG}-ranges-${request.body.id ?? "all"}`]);

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

                const skills = await fetchWithCache<{ skills: Skill[] }>("/static", requestBody, [`${CACHE_TAG}-skills-${request.body.id ?? "all"}`]);

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
                // Only cache GET requests (without trust parameter)
                const useCache = request.body.trust === undefined;

                let trust;
                if (useCache) {
                    trust = await fetchWithCache<{ trust: number | null }>("/static", requestBody, [`${CACHE_TAG}-trust-${request.body.id ?? "all"}`]);
                } else {
                    // For updates, bypass cache
                    const response = await fetch(`${env.BACKEND_URL}/static`, {
                        method: "POST",
                        headers: {
                            "Content-Type": "application/json",
                        },
                        body: JSON.stringify(requestBody),
                    });
                    trust = (await response.json()) as { trust: number | null };
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

                const handbook = await fetchWithCache<{ handbook: string }>("/static", requestBody, [`${CACHE_TAG}-handbook-${request.body.id ?? "all"}`]);

                response.writeHead(200, { "Content-Type": "application/json" });
                response.write(
                    JSON.stringify({
                        data: handbook.handbook,
                    }),
                );
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
        type: "materials" | "modules" | "operators" | "ranges" | "skills" | "trust" | "handbook";
        id?: string;
        method?: string;
        trust?: number;
    };
}
