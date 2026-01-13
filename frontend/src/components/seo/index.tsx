import Head from "next/head";

const SITE_NAME = "myrtle.moe";
const SITE_URL = "https://myrtle.moe";
const DEFAULT_DESCRIPTION = "Your comprehensive Arknights companion - DPS calculator, operator database, tier lists, recruitment calculator, account sync, and leaderboards.";
const DEFAULT_IMAGE = `${SITE_URL}/og-image.png`;

export interface SEOProps {
    /** Page title - will be appended with site name */
    title: string;
    /** Meta description for the page */
    description?: string;
    /** Canonical URL path (without domain) */
    path?: string;
    /** OpenGraph image URL */
    image?: string;
    /** OpenGraph type - defaults to "website" */
    type?: "website" | "article" | "profile";
    /** Disable indexing for this page */
    noIndex?: boolean;
    /** Additional keywords for the page */
    keywords?: string[];
    /** Article published date (for type="article") */
    publishedTime?: string;
    /** Article modified date (for type="article") */
    modifiedTime?: string;
}

/**
 * SEO component for consistent meta tags across all pages.
 *
 * Usage:
 * ```tsx
 * <SEO
 *   title="DPS Calculator"
 *   description="Calculate and compare operator DPS in Arknights."
 *   path="/tools/dps"
 * />
 * ```
 */
export function SEO({ title, description = DEFAULT_DESCRIPTION, path = "", image = DEFAULT_IMAGE, type = "website", noIndex = false, keywords = [], publishedTime, modifiedTime }: SEOProps) {
    const fullTitle = title ? `${title} | ${SITE_NAME}` : SITE_NAME;
    const canonicalURL = `${SITE_URL}${path}`;

    // Default keywords for Arknights
    const defaultKeywords = ["Arknights", "myrtle.moe", "Arknights tools", "Arknights calculator", "Arknights operators"];
    const allKeywords = [...new Set([...defaultKeywords, ...keywords])];

    return (
        <Head>
            {/* Primary Meta Tags */}
            <title>{fullTitle}</title>
            <meta content={description} name="description" />
            <meta content={allKeywords.join(", ")} name="keywords" />
            <link href={canonicalURL} rel="canonical" />

            {/* Robots */}
            {noIndex && <meta content="noindex, nofollow" name="robots" />}

            {/* Open Graph / Facebook */}
            <meta content={type} property="og:type" />
            <meta content={canonicalURL} property="og:url" />
            <meta content={fullTitle} property="og:title" />
            <meta content={description} property="og:description" />
            <meta content={image} property="og:image" />
            <meta content={SITE_NAME} property="og:site_name" />
            <meta content="en_US" property="og:locale" />

            {/* Article specific (for blog posts, changelogs, etc.) */}
            {type === "article" && publishedTime && <meta content={publishedTime} property="article:published_time" />}
            {type === "article" && modifiedTime && <meta content={modifiedTime} property="article:modified_time" />}

            {/* Twitter */}
            <meta content="summary_large_image" property="twitter:card" />
            <meta content={canonicalURL} property="twitter:url" />
            <meta content={fullTitle} property="twitter:title" />
            <meta content={description} property="twitter:description" />
            <meta content={image} property="twitter:image" />

            {/* Theme color */}
            <meta content="#1a1a2e" name="theme-color" />

            {/* Favicon - browsers will find these in public folder */}
            <link href="/favicon.ico" rel="icon" type="image/x-icon" />
        </Head>
    );
}

// Re-export for convenience
export default SEO;
