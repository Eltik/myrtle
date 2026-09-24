import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "common";

export const messages = {
    "footer.siteLinks": {
        text: "Site links",
        description: "Accessible name of the footer <nav> landmark holding the changelog, legal and community links.",
    },
    "footer.changelog": {
        text: "Changelog",
        description: "Footer link to the site's release notes.",
    },
    "footer.terms": {
        text: "Terms",
        description: "Footer link to the terms of service. Short by design; it sits in a single row of small links.",
    },
    "footer.privacy": {
        text: "Privacy",
        description: "Footer link to the privacy policy. Short by design; it sits in a single row of small links.",
    },
    "footer.discord": {
        text: "Discord",
        description: "Footer link to the community chat. 'Discord' is a product name and stays as-is.",
    },
    "footer.donate": {
        text: "Donate",
        description: "Footer link to the donation page. A verb.",
    },
    "footer.disclaimer": {
        text: "Not affiliated with Hypergryph or Yostar. Game data and assets are property of their respective owners.",
        description: "Legal disclaimer in the footer. 'Hypergryph' and 'Yostar' are company names and stay as-is.",
    },
    "footer.fontCredit": {
        text: "Terra script fonts: Sarkaz by lhclbt ({endfield}, {ccLicence}, unmodified); Sami and Aegir from {samigirian} by Siphercase ({oflLicence}, modified). Reader typeface {openDyslexic} by Abbie Gonzalez ({odLicence}, unmodified).",
        description:
            "One-line font attribution in the footer. `{endfield}`, `{samigirian}` and `{openDyslexic}` are links carrying the repository and typeface names Endfield_Font, Samigirian and OpenDyslexic; `{ccLicence}`, `{oflLicence}` and `{odLicence}` are links carrying the licence names. `Sarkaz`, `Sami`, `Aegir`, `lhclbt`, `Siphercase` and `Abbie Gonzalez` are names and stay as-is.",
    },
    "footer.builtOn": {
        text: "built on TanStack Start · COSS UI",
        description: "Credit line in the footer. Lowercase on purpose. 'TanStack Start' and 'COSS UI' are product names and stay as-is; only 'built on' is translatable.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
