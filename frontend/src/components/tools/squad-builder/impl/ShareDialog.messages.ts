import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "squad.share.title": {
        text: "Share this squad",
        description: "Title of the Squad Builder's share dialog.",
    },
    "squad.share.link": {
        text: "Link",
        description: "Heading of the share-link section.",
    },
    "squad.share.linkDesc": {
        text: "The squad lives in the link. Nothing is stored.",
        description: "Explainer under the Link heading: the link itself carries the squad, the site keeps no copy.",
    },
    "squad.share.linkField": {
        text: "Share link",
        description: "Accessible name of the read-only field holding the link.",
    },
    "squad.share.builds": {
        text: "Include builds",
        description: "Switch label: put each operator's build in the link.",
    },
    "squad.share.buildsDesc": {
        text: "Promotion, level, mastery and module rank. Off shares operators, skills and modules only.",
        description: "Explainer under the Include builds switch.",
    },
    "squad.share.copyLink": {
        text: "Copy link",
        description: "Button that copies the share link.",
    },
    "squad.share.copied": {
        text: "Link copied",
        description: "Toast after the share link was copied.",
    },
    "squad.share.copyFailed": {
        text: "Couldn't copy. Select the link and copy it yourself.",
        description: "Toast when the browser refused to copy the link.",
    },
    "squad.share.image": {
        text: "Image",
        description: "Heading of the share-image section.",
    },
    "squad.share.imageDesc": {
        text: "A picture of the squad to post anywhere.",
        description: "Explainer under the Image heading.",
    },
    "squad.share.imageAlt": {
        text: "Image of the squad {name}",
        description: "Alt text of the share image preview. {name} is the squad's name.",
    },
    "squad.share.rendering": {
        text: "Drawing the image...",
        description: "Shown while the share image renders.",
    },
    "squad.share.renderFailed": {
        text: "Couldn't draw the image.",
        description: "Shown when the share image failed to render.",
    },
    "squad.share.copyImage": {
        text: "Copy image",
        description: "Button that copies the share image to the clipboard.",
    },
    "squad.share.imageCopied": {
        text: "Image copied",
        description: "Toast after the share image was copied.",
    },
    "squad.share.imageCopyFailed": {
        text: "This browser can't copy images. Save it instead.",
        description: "Toast when copying an image is not supported or was refused.",
    },
    "squad.share.saveImage": {
        text: "Save image",
        description: "Button that downloads the share image.",
    },
    "squad.share.untitled": {
        text: "My squad",
        description: "Squad name used in the image and file name when the squad has no name.",
    },
    "squad.share.leader": {
        text: "Leader",
        description: "Label on the first seat in the share image.",
    },
    "squad.share.empty": {
        text: "Empty",
        description: "Label on an empty seat in the share image.",
    },
    "squad.share.support": {
        text: "Support",
        description: "Label over the support seat in the share image.",
    },
    "squad.share.build": {
        text: "E{phase} Lv {level}",
        description: "Build line under an operator in the share image, e.g. 'E2 Lv 90'. E is Elite (promotion).",
    },
    "squad.share.facts": {
        text: "At a glance",
        description: "Heading of the facts box in the share image.",
    },
    "squad.share.fact.rarity": {
        text: "Total rarity",
        description: "Fact in the share image: the sum of every operator's star rarity.",
    },
    "squad.share.fact.topRarity": {
        text: "{stars}★ operators",
        description: "Fact in the share image: how many operators of the top rarity are in the squad ({stars} is the highest star count in the game, e.g. 6).",
    },
    "squad.share.fact.medics": {
        text: "Medics",
        description: "Fact in the share image: how many Medic-class operators are in the squad.",
    },
    "squad.share.fact.classes": {
        text: "Classes",
        description: "Fact in the share image: how many of the game's classes the squad has.",
    },
    "squad.share.fact.classesValue": {
        text: "{count} of {total}",
        description: "Value of the Classes fact, e.g. '6 of 8'; {total} is the number of classes in the game.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
