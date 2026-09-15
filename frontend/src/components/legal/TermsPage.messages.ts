import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * Keyed at section granularity, like `PrivacyPage.messages.ts`: one key per
 * heading, one per paragraph or list item. A paragraph carrying an inline link
 * stays ONE message with a `{link}` placeholder, rendered through `useRichT`;
 * only the link's own label is a separate key.
 */
export const namespace = "legal";

export const messages = {
    // Page head
    "terms.hero.title": {
        text: "Terms of Service",
        description: "Page title of the terms of service.",
    },
    "terms.hero.subtitle": {
        text: "Legal agreement governing your use of Myrtle",
        description: "Standfirst under the terms title. 'Myrtle' is this site's name.",
    },
    "terms.hero.version": {
        text: "Version {version}",
        description: "Badge giving the document's revision number. {version} is a bare version string such as '2.0'.",
    },
    "terms.hero.effective": {
        text: "Effective: {date}",
        description: "Line giving the date the terms took effect. {date} is an already-formatted date such as 'January 12, 2026'.",
    },
    "terms.hero.lastUpdated": {
        text: "Last updated: {date}",
        description: "Line giving the date the terms were last revised. {date} is an already-formatted date.",
    },
    "terms.tldr": {
        text: "By accessing or using Myrtle, you agree to be bound by these Terms of Service. Please read them carefully before continuing.",
        description: "Banner above the terms, summarising that use implies agreement.",
    },

    // Table of contents
    "terms.toc.nav": {
        text: "Table of contents",
        description: "Accessible name of the contents navigation block.",
    },
    "terms.toc.heading": {
        text: "Table of Contents",
        description: "Visible heading of the contents block. Title Case, unlike the accessible name.",
    },
    "terms.toc.acceptance": {
        text: "Acceptance of Terms",
        description: "Contents entry for section 1. Matches that section's heading without its number.",
    },
    "terms.toc.account": {
        text: "Account Registration and Security",
        description: "Contents entry for section 2.",
    },
    "terms.toc.usage": {
        text: "Acceptable Use Policy",
        description: "Contents entry for section 3.",
    },
    "terms.toc.content": {
        text: "User Content and Submissions",
        description: "Contents entry for section 4.",
    },
    "terms.toc.intellectual": {
        text: "Intellectual Property Rights",
        description: "Contents entry for section 5.",
    },
    "terms.toc.termination": {
        text: "Termination and Suspension",
        description: "Contents entry for section 6.",
    },
    "terms.toc.liability": {
        text: "Limitation of Liability",
        description: "Contents entry for section 7.",
    },
    "terms.toc.changes": {
        text: "Changes to Terms",
        description: "Contents entry for section 8.",
    },
    "terms.toc.contact": {
        text: "Contact Information",
        description: "Contents entry for section 9.",
    },

    // 1. Acceptance
    "terms.s1.heading": {
        text: "1. Acceptance of Terms",
        description: "Heading of section 1. Keep the leading number.",
    },
    "terms.s1.p1": {
        text: 'By accessing, browsing, or using the Myrtle platform ("Service"), you acknowledge that you have read, understood, and agree to be bound by these Terms of Service and all applicable laws and regulations.',
        description: "First paragraph of section 1. The quoted word defines a term used throughout the document.",
    },
    "terms.s1.p2": {
        text: "If you do not agree with any part of these terms, you must not use our Service.",
        description: "Second paragraph of section 1.",
    },
    "terms.s1.p3": {
        text: 'These terms constitute a legally binding agreement between you ("User," "you," or "your") and Myrtle ("we," "us," or "our"). Your continued use of the Service signifies your acceptance of these terms and any modifications thereto.',
        description: "Third paragraph of section 1. The quoted words define terms used throughout; note the commas sit inside the quotation marks.",
    },

    // 2. Account
    "terms.s2.heading": {
        text: "2. Account Registration and Security",
        description: "Heading of section 2. Keep the leading number.",
    },
    "terms.s2.oauth.heading": {
        text: "2.1 Account Creation via Yostar OAuth",
        description: "Sub-heading 2.1. 'Yostar' is the game's publisher; OAuth is the sign-in standard.",
    },
    "terms.s2.oauth.body": {
        text: "To access account features like profile sync, tier lists, and leaderboards, you must authenticate using your Yostar account (the same account you use for Arknights). We use Yostar's email verification system - you will receive a verification code from Yostar to confirm your identity. We never see or store your Yostar password.",
        description: "Paragraph under 2.1, one paragraph.",
    },
    "terms.s2.security.heading": {
        text: "2.2 Account Security",
        description: "Sub-heading 2.2.",
    },
    "terms.s2.security.intro": {
        text: "You are responsible for:",
        description: "Lead-in above the list of the visitor's responsibilities.",
    },
    "terms.s2.security.item1": {
        text: "Maintaining the security of your Yostar account and email",
        description: "List item under 2.2.",
    },
    "terms.s2.security.item2": {
        text: "All activities that occur under your Myrtle account",
        description: "List item under 2.2.",
    },
    "terms.s2.security.item3": {
        text: "Notifying us if you believe your account has been compromised",
        description: "List item under 2.2.",
    },
    "terms.s2.eligibility.heading": {
        text: "2.3 Account Eligibility",
        description: "Sub-heading 2.3.",
    },
    "terms.s2.eligibility.body": {
        text: "You must be at least 13 years old to use this Service. By creating an account, you represent that you meet this age requirement and have the legal capacity to enter into these Terms. You must also have a valid Yostar account with Arknights.",
        description: "Paragraph under 2.3.",
    },
    "terms.s2.sync.heading": {
        text: "2.4 Game Data Sync",
        description: "Sub-heading 2.4.",
    },
    "terms.s2.sync.body": {
        text: "When you sync your account, we fetch your Arknights game data (operator roster, stage progress, etc.) from Yostar's servers. This data is stored on our servers to power features like your profile page, account scoring, and leaderboards. You can re-sync at any time to update your data, or delete your account to remove all stored data.",
        description: "Paragraph under 2.4, one paragraph. 'Roster' is the set of operators the player owns.",
    },

    // 3. Usage
    "terms.s3.heading": {
        text: "3. Acceptable Use Policy",
        description: "Heading of section 3. Keep the leading number.",
    },
    "terms.s3.intro": {
        text: "You agree not to engage in any of the following prohibited activities:",
        description: "Lead-in above the list of prohibited activities.",
    },
    "terms.s3.item1": {
        text: "Using the Service for any illegal or unauthorized purpose",
        description: "Prohibited-activity list item.",
    },
    "terms.s3.item2": {
        text: "Attempting to gain unauthorized access to any portion of the Service or related systems",
        description: "Prohibited-activity list item.",
    },
    "terms.s3.item3": {
        text: "Interfering with or disrupting the Service or servers connected to the Service",
        description: "Prohibited-activity list item.",
    },
    "terms.s3.item4": {
        text: "Uploading or transmitting viruses, malware, or any other malicious code",
        description: "Prohibited-activity list item.",
    },
    "terms.s3.item5": {
        text: "Scraping, crawling, or using automated systems to extract data without permission",
        description: "Prohibited-activity list item.",
    },
    "terms.s3.item6": {
        text: "Impersonating any person or entity, or falsely stating your affiliation with any person or entity",
        description: "Prohibited-activity list item.",
    },
    "terms.s3.item7": {
        text: "Harassing, threatening, or intimidating other users",
        description: "Prohibited-activity list item.",
    },
    "terms.s3.item8": {
        text: "Violating any applicable local, state, national, or international law",
        description: "Prohibited-activity list item.",
    },
    "terms.s3.outro": {
        text: "We reserve the right to investigate and prosecute violations of any of the above to the fullest extent of the law.",
        description: "Closing paragraph of section 3.",
    },

    // 4. Content
    "terms.s4.heading": {
        text: "4. User Content and Submissions",
        description: "Heading of section 4. Keep the leading number.",
    },
    "terms.s4.p1": {
        text: 'Our Service allows you to create and share content such as tier lists ("User Content"). By creating User Content, you grant us a worldwide, non-exclusive, royalty-free license to display and distribute such content in connection with the Service.',
        description: "Opening paragraph of section 4. The quoted phrase defines a term used throughout.",
    },
    "terms.s4.tierLists.heading": {
        text: "4.1 Tier Lists",
        description: "Sub-heading 4.1. A tier list ranks operators into tiers.",
    },
    "terms.s4.tierLists.body": {
        text: "You can create tier lists with full version control - every edit is logged and previous versions are preserved. You control the visibility (public, private, or shared with specific users) and can set permissions for who can edit your tier lists.",
        description: "Paragraph under 4.1.",
    },
    "terms.s4.guidelines.heading": {
        text: "4.2 Content Guidelines",
        description: "Sub-heading 4.2.",
    },
    "terms.s4.guidelines.intro": {
        text: "You represent and warrant that your User Content:",
        description: "Lead-in above the content-warranty list.",
    },
    "terms.s4.guidelines.item1": {
        text: "Is your original creation or you have the right to share it",
        description: "Content-warranty list item.",
    },
    "terms.s4.guidelines.item2": {
        text: "Does not contain offensive, harmful, or inappropriate material",
        description: "Content-warranty list item.",
    },
    "terms.s4.guidelines.item3": {
        text: "Does not impersonate other users or misrepresent your identity",
        description: "Content-warranty list item.",
    },
    "terms.s4.outro": {
        text: "We reserve the right to remove any User Content that violates these Terms or is otherwise objectionable at our sole discretion.",
        description: "Closing paragraph of section 4.",
    },

    // 5. Intellectual property
    "terms.s5.heading": {
        text: "5. Intellectual Property Rights",
        description: "Heading of section 5. Keep the leading number.",
    },
    "terms.s5.code.heading": {
        text: "5.1 Myrtle Code and Content",
        description: "Sub-heading 5.1.",
    },
    "terms.s5.code.body": {
        text: "The Myrtle platform code is open source and available on GitHub. Original features, designs, and documentation created for Myrtle are provided under applicable open source licenses. See our {link} for specific license terms.",
        description: "Paragraph under 5.1. {link} is a link to the source repository, labelled by terms.s5.code.link; move it wherever the sentence needs it.",
    },
    "terms.s5.code.link": {
        text: "GitHub repository",
        description: "Text of the link substituted into terms.s5.code.body as {link}. 'GitHub' is a product name and stays as-is.",
    },
    "terms.s5.assets.heading": {
        text: "5.2 Arknights Assets and Content",
        description: "Sub-heading 5.2.",
    },
    "terms.s5.assets.p1": {
        text: "Arknights and all related assets, characters, operator data, artwork, voice lines, and imagery are the property of Hypergryph Network Technology Co., Ltd. and Yostar Limited. This Service is a fan-made project and is not officially affiliated with, endorsed by, or sponsored by Hypergryph or Yostar.",
        description: "First paragraph under 5.2. The two company names are legal entities and must not be altered.",
    },
    "terms.s5.assets.p2": {
        text: "We use game assets under fair use for the purpose of providing game companion tools to the community. All operator data, images, Spine animations, and other game content displayed on Myrtle are sourced from the official Arknights game files.",
        description: "Second paragraph under 5.2. 'Spine' is the animation runtime's name and stays as-is.",
    },
    "terms.s5.thirdParty.heading": {
        text: "5.3 Third-Party Content",
        description: "Sub-heading 5.3.",
    },
    "terms.s5.thirdParty.body": {
        text: "Our asset processing uses FlatBuffers schemas from the {link} project (MooncellWiki). DPS calculator implementations are based on community research and reference calculations.",
        description: "Paragraph under 5.3. {link} is a link to the OpenArknightsFBS project, labelled by terms.s5.thirdParty.link; move it wherever the sentence needs it. 'FlatBuffers' is a serialisation format, 'MooncellWiki' is a community wiki's name, and DPS is damage per second.",
    },
    "terms.s5.thirdParty.link": {
        text: "OpenArknightsFBS",
        description: "Text of the link substituted into terms.s5.thirdParty.body as {link}. A project name, stays as-is.",
    },

    // 6. Termination
    "terms.s6.heading": {
        text: "6. Termination and Suspension",
        description: "Heading of section 6. Keep the leading number.",
    },
    "terms.s6.p1": {
        text: "We may terminate or suspend your account and access to the Service immediately, without prior notice or liability, for any reason, including but not limited to breach of these Terms.",
        description: "First paragraph of section 6.",
    },
    "terms.s6.p2": {
        text: "Upon termination, your right to use the Service will immediately cease. If you wish to terminate your account, you may discontinue using the Service and contact us to request account deletion.",
        description: "Second paragraph of section 6.",
    },
    "terms.s6.p3": {
        text: "All provisions of these Terms which by their nature should survive termination shall survive, including ownership provisions, warranty disclaimers, and limitations of liability.",
        description: "Third paragraph of section 6.",
    },

    // 7. Liability
    "terms.s7.heading": {
        text: "7. Limitation of Liability",
        description: "Heading of section 7. Keep the leading number.",
    },
    "terms.s7.p1": {
        text: 'The Service is provided on an "AS IS" and "AS AVAILABLE" basis without warranties of any kind, either express or implied. We do not warrant that the Service will be uninterrupted, secure, or error-free.',
        description: "First paragraph of section 7. The quoted capitals are standard warranty-disclaimer wording.",
    },
    "terms.s7.p2": {
        text: "To the maximum extent permitted by law, Myrtle shall not be liable for any indirect, incidental, special, consequential, or punitive damages, including but not limited to loss of profits, data, or other intangible losses, resulting from:",
        description: "Second paragraph of section 7, which runs into the list below it.",
    },
    "terms.s7.item1": {
        text: "Your access to or use of (or inability to access or use) the Service",
        description: "Liability list item.",
    },
    "terms.s7.item2": {
        text: "Any unauthorized access to or use of our servers and/or any personal information stored therein",
        description: "Liability list item.",
    },
    "terms.s7.item3": {
        text: "Any interruption or cessation of transmission to or from the Service",
        description: "Liability list item.",
    },
    "terms.s7.item4": {
        text: "Any bugs, viruses, or other harmful code that may be transmitted through the Service",
        description: "Liability list item.",
    },

    // 8. Changes
    "terms.s8.heading": {
        text: "8. Changes to Terms",
        description: "Heading of section 8. Keep the leading number.",
    },
    "terms.s8.p1": {
        text: "We reserve the right to modify or replace these Terms at any time at our sole discretion. If a revision is material, we will provide at least 30 days' notice prior to any new terms taking effect.",
        description: "First paragraph of section 8.",
    },
    "terms.s8.p2": {
        text: "What constitutes a material change will be determined at our sole discretion. By continuing to access or use our Service after revisions become effective, you agree to be bound by the revised terms.",
        description: "Second paragraph of section 8.",
    },

    // 9. Contact
    "terms.s9.heading": {
        text: "9. Contact Information",
        description: "Heading of section 9. Keep the leading number.",
    },
    "terms.s9.intro": {
        text: "If you have any questions about these Terms, please contact us:",
        description: "Lead-in above the contact card.",
    },
    "terms.s9.gdpr.label": {
        text: "Privacy & GDPR requests",
        description: "Label over the privacy mailbox. GDPR is the EU data-protection regulation; the ampersand is literal.",
    },
    "terms.s9.gdpr.note": {
        text: "For access, rectification, erasure, portability, or objection requests under the GDPR (or comparable data-protection laws).",
        description: "Caption under the privacy mailbox, naming the GDPR rights it serves.",
    },
    "terms.s9.general.label": {
        text: "General contact",
        description: "Label over the general mailbox.",
    },
    "terms.s9.source.label": {
        text: "Source code",
        description: "Label over the link to the source repository.",
    },
    "terms.s9.inApp.label": {
        text: "In-app",
        description: "Label over the link into this site.",
    },
    "terms.s9.inApp.link": {
        text: "Manage your account in settings",
        description: "Text of the in-app link.",
    },
    "terms.related.privacy": {
        text: "Privacy Policy",
        description: "Cross-link at the foot of the terms, pointing at the privacy page. Use the same wording as that page's own title.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
