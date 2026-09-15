import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * Keyed at section granularity: one key per heading, one per paragraph or
 * list item. A paragraph is the smallest unit a translator can render
 * coherently - clause order differs between languages, so a sentence-level
 * split would produce nonsense.
 *
 * Where a paragraph carries an inline `<a>` or `<strong>`, it stays ONE
 * message with a `{placeholder}` for the element, rendered through `useRichT`.
 * The element's own text (a link label, a bolded path) keeps its own key so a
 * translator can render or keep it, but the sentence around it is never split
 * into fragments - a translator must be able to move the element.
 */
export const namespace = "legal";

export const messages = {
    // Hero
    "privacy.hero.title": {
        text: "Privacy Policy",
        description: "Page title of the privacy policy.",
    },
    "privacy.hero.subtitle": {
        text: "Your privacy matters. Here's how we protect and handle your personal information.",
        description: "Standfirst under the privacy policy title.",
    },
    "privacy.hero.effective": {
        text: "Effective: {date}",
        description: "Badge giving the date the policy took effect. {date} is an already-formatted date such as 'Jan 12, 2026'.",
    },
    "privacy.hero.version": {
        text: "Version {version}",
        description: "Badge giving the policy's revision number. {version} is a bare version string such as '2.0'.",
    },

    // TL;DR
    "privacy.tldr.title": {
        text: "TL;DR - Quick Summary",
        description: "Title of the summary banner. 'TL;DR' is internet shorthand for 'too long; didn't read'.",
    },
    "privacy.tldr.body": {
        text: "We use Yostar OAuth to sync your game data - we never see your password and don't store your email address. We do keep your Yostar token, encrypted, so re-syncing doesn't need a new email code every time; you can delete it yourself any time from Settings. We collect only what's needed to provide our tools. We never sell your information. You control your profile visibility and leaderboard participation. You can delete your account and all data anytime.",
        description: "The whole summary banner, one paragraph. 'Yostar' is the game's publisher; OAuth is the sign-in standard.",
    },

    // Principles
    "privacy.principles.heading": {
        text: "Our Privacy Principles",
        description: "Heading over the three principle tiles.",
    },
    "privacy.principles.minimal.title": {
        text: "Minimal Collection",
        description: "Title of the first principle tile.",
    },
    "privacy.principles.minimal.desc": {
        text: "We only collect what we absolutely need to provide our services.",
        description: "Body of the Minimal Collection tile.",
    },
    "privacy.principles.transparency.title": {
        text: "Full Transparency",
        description: "Title of the second principle tile.",
    },
    "privacy.principles.transparency.desc": {
        text: "We're upfront about what data we collect and why we collect it.",
        description: "Body of the Full Transparency tile.",
    },
    "privacy.principles.control.title": {
        text: "Your Control",
        description: "Title of the third principle tile.",
    },
    "privacy.principles.control.desc": {
        text: "You decide what's visible and can delete your data at any time.",
        description: "Body of the Your Control tile.",
    },

    // Yostar OAuth
    "privacy.yostar.heading": {
        text: "Yostar Account Integration",
        description: "Section heading for the game-account sync section.",
    },
    "privacy.yostar.subheading": {
        text: "How we sync your Arknights game data",
        description: "Subheading under Yostar Account Integration.",
    },
    "privacy.yostar.intro": {
        text: "Myrtle uses Yostar's official OAuth system to sync your Arknights account data. Here's how it works:",
        description: "Opening paragraph of the Yostar section. 'Myrtle' is this site's name.",
    },
    "privacy.yostar.auth.title": {
        text: "How Authentication Works",
        description: "Callout title inside the Yostar section.",
    },
    "privacy.yostar.auth.body": {
        text: "When you log in, you enter your email address in our app. Yostar sends a verification code directly to your email. You enter that code to authenticate. We never see or store your Yostar password - authentication is handled entirely by Yostar's servers. Your email address is only used during the authentication process and is not stored in our database.",
        description: "Callout body explaining the sign-in flow, one paragraph.",
    },
    "privacy.yostar.receive.title": {
        text: "What We Receive from Yostar",
        description: "Callout title inside the Yostar section.",
    },
    "privacy.yostar.receive.p1": {
        text: "Upon successful authentication, Yostar provides us with a session token that allows us to fetch your public game data: operator roster, levels, promotions, skill masteries, modules, stage progress, base layout, inventory, and account statistics. This is the same data visible in your in-game profile.",
        description: "First paragraph of the callout. Roster, promotions, masteries, modules and base are in-game systems.",
    },
    "privacy.yostar.receive.p2": {
        text: "We keep that token so re-syncing works without emailing you a new code every time. It is stored encrypted, never leaves our server, and is never shown to you or to anyone else. It stays until you disconnect your game account or delete your Myrtle account.",
        description: "Second paragraph of the same callout, separated from the first by a blank line.",
    },
    "privacy.yostar.revoke.body": {
        text: "We are not affiliated with Hypergryph or Yostar. We access your data through the same APIs the official game client uses. You can revoke that access yourself at any time: {path} deletes the stored token, and your synced data stays on your profile. To remove the data too, request account deletion by emailing {email}.",
        description: "Closing paragraph of the Yostar section. {path} is the bolded settings path below and {email} is the contact address, rendered as a mail link; move both wherever the sentence needs them.",
    },
    "privacy.yostar.revoke.path": {
        text: "Settings → Account & data → Disconnect game account",
        description: "The bolded navigation path through this site's own settings screen, substituted into privacy.yostar.revoke.body as {path}. Use the same wording as those settings labels; the arrows and ampersand are literal.",
    },

    // Information we collect
    "privacy.collect.heading": {
        text: "Information We Collect",
        description: "Section heading for the data-collection inventory.",
    },
    "privacy.collect.subheading": {
        text: "Understanding what data flows through our system",
        description: "Subheading under Information We Collect.",
    },
    "privacy.collect.note": {
        text: "Note:",
        description: "Bolded lead-in on the two highlighted notes in this section. Includes its colon.",
    },
    "privacy.collect.account.heading": {
        text: "Account Information",
        description: "Sub-heading over the list of stored account fields.",
    },
    "privacy.collect.account.intro": {
        text: "When you create an account via Yostar OAuth, we store:",
        description: "Lead-in above the account-information list.",
    },
    "privacy.collect.account.uid.label": {
        text: "Arknights UID and nickname",
        description: "Bolded term of a list item. UID is the in-game account number.",
    },
    "privacy.collect.account.uid.body": {
        text: "- your in-game identifier and display name",
        description: "Gloss following the bolded term, on the same line. Begins with a hyphen and a space, matching the other items.",
    },
    "privacy.collect.account.settings.label": {
        text: "Profile settings",
        description: "Bolded term of a list item.",
    },
    "privacy.collect.account.settings.body": {
        text: "- your Myrtle preferences: theme, accent color, profile visibility, leaderboard opt-in, and notification settings",
        description: "Gloss following the bolded term. Begins with a hyphen and a space.",
    },
    "privacy.collect.emailNote": {
        text: "We do not store your email address. It is only used during the Yostar authentication process to receive your verification code and is not saved to our database.",
        description: "Highlighted note, following the bolded 'Note:' lead-in.",
    },
    "privacy.collect.game.heading": {
        text: "Game Data (Synced from Arknights)",
        description: "Sub-heading over the list of synced game data.",
    },
    "privacy.collect.game.intro": {
        text: "When you sync your account, we fetch and store:",
        description: "Lead-in above the game-data list.",
    },
    "privacy.collect.game.roster.label": {
        text: "Operator roster",
        description: "Bolded term of a list item: the set of operators the player owns.",
    },
    "privacy.collect.game.roster.body": {
        text: "- all operators you own, including level, promotion, trust, potential, skill levels, masteries, and equipped modules",
        description: "Gloss following the bolded term. Level, promotion, trust, potential, mastery and module are in-game systems. Begins with a hyphen and a space.",
    },
    "privacy.collect.game.stages.label": {
        text: "Stage progress",
        description: "Bolded term of a list item.",
    },
    "privacy.collect.game.stages.body": {
        text: "- mainline, sidestory, and activity stage completion status",
        description: "Gloss following the bolded term. Mainline, sidestory and activity are in-game stage categories. Begins with a hyphen and a space.",
    },
    "privacy.collect.game.roguelike.label": {
        text: "Roguelike & Sandbox progress",
        description: "Bolded term of a list item. The ampersand is literal.",
    },
    "privacy.collect.game.roguelike.body": {
        text: "- Integrated Strategies themes, endings, buffs, and Reclamation Algorithm data",
        description: "Gloss following the bolded term. 'Integrated Strategies' and 'Reclamation Algorithm' are in-game mode names; keep the game's own names. Begins with a hyphen and a space.",
    },
    "privacy.collect.game.base.label": {
        text: "Base layout",
        description: "Bolded term of a list item: the player's home installation.",
    },
    "privacy.collect.game.base.body": {
        text: "- RIIC building configuration and efficiency data",
        description: "Gloss following the bolded term. 'RIIC' is the in-game base system's name. Begins with a hyphen and a space.",
    },
    "privacy.collect.game.medals.label": {
        text: "Medals and achievements",
        description: "Bolded term of a list item.",
    },
    "privacy.collect.game.medals.body": {
        text: "- your collection of in-game medals",
        description: "Gloss following the bolded term. Begins with a hyphen and a space.",
    },
    "privacy.collect.saved.heading": {
        text: "Saved Configurations",
        description: "Sub-heading over the list of things the visitor has saved on the site.",
    },
    "privacy.collect.saved.dps.label": {
        text: "DPS calculator configurations",
        description: "Bolded term of a list item. DPS is damage per second.",
    },
    "privacy.collect.saved.dps.body": {
        text: "- saved operator setups and comparison configurations",
        description: "Gloss following the bolded term. Begins with a hyphen and a space.",
    },
    "privacy.collect.technical.heading": {
        text: "Technical Data",
        description: "Sub-heading over the list of diagnostic data.",
    },
    "privacy.collect.technical.logs.label": {
        text: "Error logs",
        description: "Bolded term of a list item.",
    },
    "privacy.collect.technical.logs.body": {
        text: "- diagnostic data to help us fix bugs, stored temporarily",
        description: "Gloss following the bolded term. Begins with a hyphen and a space.",
    },
    "privacy.collect.ipNote": {
        text: "Your IP address is not collected or stored. All API requests are routed through our server, which handles communication with our backend internally. Rate limiting is applied at the server level, not based on individual user IPs.",
        description: "Highlighted note, following the bolded 'Note:' lead-in.",
    },

    // How we use it
    "privacy.use.heading": {
        text: "How We Use Your Information",
        description: "Section heading for the purposes of processing.",
    },
    "privacy.use.subheading": {
        text: "The purposes behind our data collection",
        description: "Subheading under How We Use Your Information.",
    },
    "privacy.use.service.summary": {
        text: "Service Provision",
        description: "Summary line of an expandable panel; clicking it reveals the paragraph below.",
    },
    "privacy.use.service.body": {
        text: "To operate and maintain your account, sync your data across devices, and provide the core functionality of our tools and calculators. This includes displaying your operator roster, calculating account scores, and enabling profile features.",
        description: "Body of the Service Provision panel.",
    },
    "privacy.use.leaderboards.summary": {
        text: "Leaderboards & Community Features",
        description: "Summary line of an expandable panel. The ampersand is literal.",
    },
    "privacy.use.leaderboards.body": {
        text: "If you opt in to public visibility, your account scores and rankings may appear on our leaderboards. This allows the community to compare account progress across multiple dimensions (operators, stages, roguelike, sandbox, medals, base). You can opt out at any time in your settings.",
        description: "Body of the Leaderboards panel.",
    },
    "privacy.use.improvement.summary": {
        text: "Improvement & Development",
        description: "Summary line of an expandable panel. The ampersand is literal.",
    },
    "privacy.use.improvement.body": {
        text: "To understand how users interact with features, identify bugs, optimize performance, and develop new functionality based on usage patterns. We use aggregated, anonymized data for these purposes.",
        description: "Body of the Improvement panel.",
    },
    "privacy.use.security.summary": {
        text: "Security & Abuse Prevention",
        description: "Summary line of an expandable panel. The ampersand is literal.",
    },
    "privacy.use.security.body": {
        text: "To protect against unauthorized access, detect suspicious activity, and maintain the integrity of our platform. Rate limiting is applied at the server level to prevent abuse - your individual IP address is not tracked or stored.",
        description: "Body of the Security panel.",
    },

    // Settings & visibility
    "privacy.settings.heading": {
        text: "User Settings & Profile Visibility",
        description: "Section heading for the visibility controls. The ampersand is literal.",
    },
    "privacy.settings.subheading": {
        text: "Control how your information is displayed",
        description: "Subheading under User Settings & Profile Visibility.",
    },
    "privacy.settings.intro": {
        text: "Your settings page gives you full control over how your data is shared with others:",
        description: "Opening paragraph of the settings section.",
    },
    "privacy.settings.visibility.title": {
        text: "Profile Visibility",
        description: "Callout title for the profile-visibility control.",
    },
    "privacy.settings.visibility.body": {
        text: "Choose whether your profile is public (anyone can view), friends-only, or completely private.",
        description: "Callout body for the profile-visibility control.",
    },
    "privacy.settings.leaderboard.title": {
        text: "Leaderboard Participation",
        description: "Callout title for the leaderboard opt-in.",
    },
    "privacy.settings.leaderboard.body": {
        text: "Opt in or out of appearing on public leaderboards. Your scores are still calculated but won't be displayed publicly if you opt out.",
        description: "Callout body for the leaderboard opt-in.",
    },
    "privacy.settings.public.title": {
        text: "What's Publicly Visible (when profile is public)",
        description: "Callout title over the list of publicly visible fields.",
    },
    "privacy.settings.public.nickname": {
        text: "Your Arknights nickname and UID",
        description: "List item under What's Publicly Visible. UID is the in-game account number.",
    },
    "privacy.settings.public.roster": {
        text: "Your operator roster and account scores",
        description: "List item under What's Publicly Visible.",
    },
    "privacy.settings.public.rankings": {
        text: "Your leaderboard rankings (if opted in)",
        description: "List item under What's Publicly Visible.",
    },
    "privacy.settings.public.avatar": {
        text: "Your profile avatar and selected assistant",
        description: "List item under What's Publicly Visible. 'Assistant' is the game's term for the operator shown on the home screen.",
    },
    "privacy.settings.private.title": {
        text: "What's Always Private",
        description: "Callout title over the list of fields that are never published.",
    },
    "privacy.settings.private.tokens": {
        text: "Your authentication tokens and session data",
        description: "List item under What's Always Private.",
    },
    "privacy.settings.private.settings": {
        text: "Your settings and preferences",
        description: "List item under What's Always Private.",
    },
    "privacy.settings.private.dps": {
        text: "Your saved DPS calculator configurations",
        description: "List item under What's Always Private. DPS is damage per second.",
    },
    "privacy.settings.private.email": {
        text: "Your email and authentication methods",
        description: "List item under What's Always Private.",
    },

    // Security
    "privacy.security.heading": {
        text: "Data Security & Storage",
        description: "Section heading for the security measures. The ampersand is literal.",
    },
    "privacy.security.subheading": {
        text: "How we protect your information",
        description: "Subheading under Data Security & Storage.",
    },
    "privacy.security.intro": {
        text: "We implement security measures to protect your personal information:",
        description: "Opening paragraph of the security section.",
    },
    "privacy.security.encryption.title": {
        text: "Encryption",
        description: "Callout title in the security grid.",
    },
    "privacy.security.encryption.body": {
        text: "All data transmitted over HTTPS with TLS encryption",
        description: "Callout body in the security grid. No closing full stop, matching its neighbours.",
    },
    "privacy.security.jwt.title": {
        text: "JWT Authentication",
        description: "Callout title in the security grid. 'JWT' is the JSON Web Token standard and stays as-is.",
    },
    "privacy.security.jwt.body": {
        text: "Secure token-based sessions with automatic expiration",
        description: "Callout body in the security grid. No closing full stop, matching its neighbours.",
    },
    "privacy.security.rateLimit.title": {
        text: "Server-Side Rate Limiting",
        description: "Callout title in the security grid.",
    },
    "privacy.security.rateLimit.body": {
        text: "Rate limits applied at server level without tracking user IPs",
        description: "Callout body in the security grid. No closing full stop, matching its neighbours.",
    },
    "privacy.security.redis.title": {
        text: "Redis Caching",
        description: "Callout title in the security grid. 'Redis' is a product name and stays as-is.",
    },
    "privacy.security.redis.body": {
        text: "Game data cached with a 1-hour TTL for performance. Your Yostar token is not kept here - it is stored encrypted in the database.",
        description: "Callout body in the security grid. 'TTL' is time-to-live, a cache expiry.",
    },
    "privacy.security.database.title": {
        text: "Database",
        description: "Callout title below the security grid.",
    },
    "privacy.security.database.body": {
        text: "Your data is stored in PostgreSQL with proper constraints and transactions. Game data is fetched fresh from Yostar's servers when you sync - we store a copy so you can access your profile and leaderboard features without re-authenticating.",
        description: "Callout body below the security grid. 'PostgreSQL' is a product name and stays as-is.",
    },
    "privacy.security.openSource": {
        text: "While we strive to protect your information, no method of transmission over the internet is 100% secure. Our code is open source on {link}, so you can review our security practices.",
        description: "Closing paragraph of the security section. {link} is a link to the source repository, labelled by privacy.security.githubLink; move it wherever the sentence needs it.",
    },
    "privacy.security.githubLink": {
        text: "GitHub",
        description: "Text of the link substituted into privacy.security.openSource as {link}. Product name, stays as-is.",
    },

    // Rights
    "privacy.rights.heading": {
        text: "Your Rights & Choices",
        description: "Section heading for the visitor's data rights. The ampersand is literal.",
    },
    "privacy.rights.subheading": {
        text: "Control over your personal data",
        description: "Subheading under Your Rights & Choices.",
    },
    "privacy.rights.intro": {
        text: "You have the following rights regarding your personal information:",
        description: "Opening paragraph of the rights section.",
    },
    "privacy.rights.access.summary": {
        text: "Access & Export",
        description: "Summary line of an expandable panel. The ampersand is literal.",
    },
    "privacy.rights.access.body": {
        text: "Request a copy of all data we hold about you in a portable format. You can view most of your data directly on your profile page, including your synced game data, scores, and settings.",
        description: "Body of the Access & Export panel.",
    },
    "privacy.rights.correction.summary": {
        text: "Correction & Re-sync",
        description: "Summary line of an expandable panel. The ampersand is literal.",
    },
    "privacy.rights.correction.body": {
        text: "Update your preferences through your account settings. If your game data is out of date, you can re-sync at any time to fetch the latest information from Yostar's servers.",
        description: "Body of the Correction & Re-sync panel.",
    },
    "privacy.rights.visibility.summary": {
        text: "Visibility Control",
        description: "Summary line of an expandable panel.",
    },
    "privacy.rights.visibility.body": {
        text: "Choose who can see your profile and roster (public, friends-only, or private). Control whether you appear on public leaderboards. All visibility settings can be changed at any time in your settings page.",
        description: "Body of the Visibility Control panel.",
    },
    "privacy.rights.deletion.summary": {
        text: "Account Deletion",
        description: "Summary line of an expandable panel.",
    },
    "privacy.rights.deletion.body": {
        text: "Request permanent deletion of your account and all associated data. This removes your profile, synced game data, scores, and any saved configurations. This action cannot be undone. To request deletion, email {email}.",
        description: "Body of the Account Deletion panel. {email} is the contact address, rendered as a mail link; move it wherever the sentence needs it.",
    },

    // Third parties
    "privacy.thirdParty.heading": {
        text: "Third-Party Services",
        description: "Section heading for the list of outside services.",
    },
    "privacy.thirdParty.intro": {
        text: "We interact with the following third-party services:",
        description: "Lead-in above the third-party list.",
    },
    "privacy.thirdParty.yostar.label": {
        text: "Yostar / Hypergryph:",
        description: "Bolded lead-in of a list item, including its colon. Both are company names.",
    },
    "privacy.thirdParty.yostar.body": {
        text: "We use Yostar's OAuth system to authenticate you and fetch your game data. We are not affiliated with Yostar or Hypergryph - Myrtle is an independent fan project.",
        description: "Body of the Yostar / Hypergryph list item, following the bolded lead-in.",
    },
    "privacy.thirdParty.servers.label": {
        text: "Arknights game servers:",
        description: "Bolded lead-in of a list item, including its colon.",
    },
    "privacy.thirdParty.servers.body": {
        text: "We fetch game data from official servers (EN, JP, KR, CN, TW, Bilibili) to provide up-to-date operator information, assets, and your synced account data.",
        description: "Body of the game-servers list item. The server codes are identifiers and stay as-is.",
    },
    "privacy.thirdParty.hosting.label": {
        text: "Hosting infrastructure:",
        description: "Bolded lead-in of a list item, including its colon.",
    },
    "privacy.thirdParty.hosting.body": {
        text: "Our servers and databases are hosted on secure cloud infrastructure.",
        description: "Body of the hosting list item, following the bolded lead-in.",
    },
    "privacy.thirdParty.noSelling": {
        text: "We do not sell, trade, or rent your personal information to third parties. We do not run ads or use your data for marketing.",
        description: "Closing paragraph of the third-party section, set in muted text.",
    },

    // Children
    "privacy.children.heading": {
        text: "Children's Privacy",
        description: "Section heading for the minimum-age policy.",
    },
    "privacy.children.body": {
        text: "Our Service is not directed to individuals under the age of 13. We do not knowingly collect personal information from children under 13. If you become aware that a child has provided us with personal information, please contact us and we will take steps to delete such information.",
        description: "The whole minimum-age section, one paragraph.",
    },

    // Changes
    "privacy.changes.heading": {
        text: "Changes to This Policy",
        description: "Section heading for how the policy is revised.",
    },
    "privacy.changes.p1": {
        text: 'We may update this Privacy Policy from time to time to reflect changes in our practices or legal requirements. We will notify you of any material changes by posting the new policy on this page and updating the "Effective Date" at the top.',
        description: "First paragraph of the changes section. The quoted phrase names the badge at the top of this page.",
    },
    "privacy.changes.p2": {
        text: "We encourage you to review this Privacy Policy periodically. Your continued use of the Service after changes are posted constitutes your acceptance of the updated policy.",
        description: "Second paragraph of the changes section.",
    },

    // Contact
    "privacy.contact.heading": {
        text: "Contact Us",
        description: "Section heading for the contact details.",
    },
    "privacy.contact.subheading": {
        text: "Questions about your privacy?",
        description: "Subheading under Contact Us.",
    },
    "privacy.contact.getInTouch": {
        text: "Get in Touch",
        description: "Heading of the contact card.",
    },
    "privacy.contact.intro": {
        text: "If you have any questions, concerns, or requests regarding this Privacy Policy or your personal data, please contact us:",
        description: "Lead-in inside the contact card.",
    },
    "privacy.contact.gdpr.label": {
        text: "Privacy & GDPR requests",
        description: "Label over the privacy mailbox. GDPR is the EU data-protection regulation; the ampersand is literal.",
    },
    "privacy.contact.gdpr.note": {
        text: "For access, rectification, erasure, portability, or objection requests under the GDPR (or comparable data-protection laws).",
        description: "Caption under the privacy mailbox, naming the GDPR rights it serves.",
    },
    "privacy.contact.general.label": {
        text: "General contact",
        description: "Label over the general mailbox.",
    },
    "privacy.contact.source.label": {
        text: "Source code",
        description: "Label over the link to the source repository.",
    },
    "privacy.contact.inApp.label": {
        text: "In-app",
        description: "Label over the link into this site's own settings screen.",
    },
    "privacy.contact.inApp.link": {
        text: "Manage your privacy in settings",
        description: "Text of the link into the settings screen.",
    },

    // Footer CTA
    "privacy.cta.title": {
        text: "Your Privacy is Our Priority",
        description: "Heading of the closing panel.",
    },
    "privacy.cta.body": {
        text: "We're committed to transparency and giving you control over your data. If you have any questions or concerns, we're here to help.",
        description: "Body of the closing panel.",
    },
    "privacy.cta.manageSettings": {
        text: "Manage your privacy settings",
        description: "Primary button in the closing panel, linking to the settings screen.",
    },
    "privacy.cta.returnHome": {
        text: "Return home",
        description: "Secondary button in the closing panel. Sentence case here, unlike the 'Return Home' button in the shared related-documents footer.",
    },
    "privacy.related.terms": {
        text: "Terms of Service",
        description: "Cross-link at the foot of the privacy policy, pointing at the terms page. Use the same wording as that page's own title.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
