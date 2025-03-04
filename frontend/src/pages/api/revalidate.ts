import type { NextApiRequest, NextApiResponse } from "next";

// Define custom API handler for cache revalidation
export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    // Check for secret to confirm this is a valid request
    if (req.query.secret !== process.env.REVALIDATION_SECRET) {
        return res.status(401).json({ message: "Invalid token" });
    }

    try {
        // This will revalidate caches for the specified path or tag
        if (req.query.path) {
            // Revalidate specific page
            await res.revalidate(req.query.path as string);
            return res.json({ revalidated: true, path: req.query.path });
        }

        if (req.query.tag) {
            // Revalidate by cache tag (works with unstable_cache tags)
            await res.revalidate(`/${req.query.tag as string}`);
            return res.json({ revalidated: true, tag: req.query.tag });
        }

        // If no path or tag specified, revalidate the static-api cache tag
        await res.revalidate("/static-api-cache");
        return res.json({ revalidated: true });
    } catch (err) {
        console.error(err);
        // If there was an error, Next.js will continue to show the last successful generated page
        return res.status(500).send("Error revalidating cache");
    }
}
