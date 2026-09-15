import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "home";

export const messages = {
    "tierLists.kicker": {
        text: "Community",
        description: "Eyebrow label above the tier-list preview section on the landing page. Rendered uppercase.",
    },
    "tierLists.title": {
        text: "Tier lists, at a glance.",
        description: "Heading of the tier-list preview section on the landing page. A tier list ranks operators into named tiers.",
    },
    "tierLists.blurb": {
        text: "Previews from the most-watched community lists. Click any card to open the full ranking.",
        description: "Caption under the tier-list section heading on the landing page.",
    },
    "tierLists.filter.all": {
        text: "All",
        description: "First tab of the tier-list tag filter, meaning 'no tag filter'. The other tabs are tag names from the data and are not translated.",
    },
    "tierLists.error": {
        text: "Failed to load tier lists. Try refreshing the page.",
        description: "Shown in place of the tier-list cards when their request failed.",
    },
    "tierLists.empty": {
        text: "No tier lists yet. Be the first to publish one.",
        description: "Empty state shown when no community tier list has been published.",
    },
    "tierLists.browseAll": {
        text: "Browse all tier lists",
        description: "Link under the tier-list previews; opens the full tier-list index.",
    },
    "tierLists.arrowIcon": {
        text: "Right arrow",
        description: "Accessible name of the decorative arrow icon in the 'Browse all tier lists' link.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
