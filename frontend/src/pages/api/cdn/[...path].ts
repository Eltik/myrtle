import type { NextApiRequest, NextApiResponse } from "next";
import { env } from "../../../env";
import { backendFetch } from "~/lib/backend-fetch";

export const config = {
    api: {
        responseLimit: false,
    },
};

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    if (req.method !== "GET") {
        return res.status(405).json({ error: "Method not allowed" });
    }

    const isDevelopment = env.NODE_ENV === "development";

    try {
        const pathSegments = req.query.path;
        if (!pathSegments || !Array.isArray(pathSegments) || pathSegments.length === 0) {
            return res.status(400).json({ error: "Invalid path" });
        }

        const fullPath = pathSegments.join("/");

        // Route avatar and portrait paths to dedicated backend endpoints
        const avatarMatch = fullPath.match(/^avatar[s]?\/(.+)$/);
        const portraitMatch = fullPath.match(/^portrait[s]?\/(.+)$/);

        if (avatarMatch?.[1]) {
            // Strip .png extension - backend looks up by asset name
            const id = avatarMatch[1].replace(/\.png$/i, "");
            const response = await backendFetch(`/avatar/${id}`);

            if (!response.ok) {
                return res.status(response.status).json({ error: "Failed to fetch avatar" });
            }

            const buffer = Buffer.from(await response.arrayBuffer());
            const contentType = response.headers.get("Content-Type") ?? "image/png";

            res.setHeader("Content-Type", contentType);
            res.setHeader("Content-Length", buffer.length);
            if (!isDevelopment) {
                res.setHeader("Cache-Control", "public, max-age=86400, stale-while-revalidate=604800");
            }
            return res.status(200).send(buffer);
        }

        const portraitId = portraitMatch?.[1];
        if (portraitId) {
            // Strip .png extension and _1/_2 suffix - backend appends these internally
            const rawId = portraitId.replace(/\.png$/i, "").replace(/_(1\+?|2)$/, "");
            const response = await backendFetch(`/portrait/${rawId}`);

            if (!response.ok) {
                return res.status(response.status).json({ error: "Failed to fetch portrait" });
            }

            const buffer = Buffer.from(await response.arrayBuffer());
            const contentType = response.headers.get("Content-Type") ?? "image/png";

            res.setHeader("Content-Type", contentType);
            res.setHeader("Content-Length", buffer.length);
            if (!isDevelopment) {
                res.setHeader("Cache-Control", "public, max-age=86400, stale-while-revalidate=604800");
            }
            return res.status(200).send(buffer);
        }

        // Route known asset types to dedicated backend endpoints
        const skillIconMatch = fullPath.match(/^(?:skill[_-]?icons?|upk\/.*skill.*?)\/(.+?)(?:\.png)?$/i);
        const moduleIconMatch = fullPath.match(/^(?:module[_-]?icons?|upk\/.*equip_small.*?)\/(.+?)(?:\.png)?$/i);
        const moduleBigMatch = fullPath.match(/^(?:module[_-]?big|upk\/.*equip_big.*?)\/(.+?)(?:\.png)?$/i);
        const enemyIconMatch = fullPath.match(/^(?:enemy[_-]?icons?|upk\/.*icon_enemies.*?)\/(.+?)(?:\.png)?$/i);
        const itemIconMatch = fullPath.match(/^(?:item[_-]?icons?|upk\/.*ui_item.*?)\/(.+?)(?:\.png)?$/i);
        const skinPortraitMatch = fullPath.match(/^(?:skin[_-]?portraits?|upk\/.*skin_portrait.*?)\/(.+?)(?:\.png)?$/i);
        const charartMatch = fullPath.match(/^(?:chararts?|upk\/.*chararts.*?)\/(.+?)(?:\.png)?$/i);

        const assetMatch = skillIconMatch ? { route: "skill-icon", id: skillIconMatch[1] }
            : moduleIconMatch ? { route: "module-icon", id: moduleIconMatch[1] }
            : moduleBigMatch ? { route: "module-big", id: moduleBigMatch[1] }
            : enemyIconMatch ? { route: "enemy-icon", id: enemyIconMatch[1] }
            : itemIconMatch ? { route: "item-icon", id: itemIconMatch[1] }
            : skinPortraitMatch ? { route: "skin-portrait", id: skinPortraitMatch[1] }
            : charartMatch ? { route: "charart", id: charartMatch[1] }
            : null;

        if (assetMatch?.id) {
            const response = await backendFetch(`/${assetMatch.route}/${assetMatch.id}`);
            if (response.ok) {
                const buffer = Buffer.from(await response.arrayBuffer());
                const contentType = response.headers.get("Content-Type") ?? "image/png";
                res.setHeader("Content-Type", contentType);
                res.setHeader("Content-Length", buffer.length);
                if (!isDevelopment) {
                    res.setHeader("Cache-Control", "public, max-age=86400, stale-while-revalidate=604800");
                }
                return res.status(200).send(buffer);
            }
            // Fall through to generic assets endpoint if dedicated route fails
        }

        // Fallback: proxy to generic /assets/ endpoint
        // Map legacy "upk/" prefix to "textures/" which is the actual directory name in v3
        // Encode each segment to handle special chars like # in skin names
        let mappedSegments = [...pathSegments];
        if (mappedSegments[0] === "upk") {
            mappedSegments[0] = "textures";
        }
        const assetPath = mappedSegments.map((s) => encodeURIComponent(s)).join("/");
        const response = await backendFetch(`/assets/${assetPath}`);

        if (!response.ok) {
            return res.status(response.status).json({ error: "Asset not found" });
        }

        const buffer = Buffer.from(await response.arrayBuffer());
        const contentType = response.headers.get("Content-Type") ?? "application/octet-stream";
        const etag = response.headers.get("ETag");
        const lastModified = response.headers.get("Last-Modified");

        res.setHeader("Content-Type", contentType);
        res.setHeader("Content-Length", buffer.length);
        if (etag) res.setHeader("ETag", etag);
        if (lastModified) res.setHeader("Last-Modified", lastModified);

        if (isDevelopment) {
            res.setHeader("Cache-Control", "no-store, max-age=0");
        } else {
            res.setHeader("Cache-Control", "public, max-age=86400, stale-while-revalidate=604800");
        }

        return res.status(200).send(buffer);
    } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        console.error("Error proxying to CDN:", { message: errorMessage, url: req.url });
        res.status(500).json({ error: "Internal server error" });
    }
}
