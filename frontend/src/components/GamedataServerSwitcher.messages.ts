import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "common";

export const messages = {
    "gamedataServer.label": {
        text: "Game text",
        description: "A noun phrase naming what the game-data server picker selects: which Arknights client's operator names, skill descriptions and story text the site shows. Heading of the picker in the header's language menu and in appearance settings.",
    },
    "gamedataServer.choose": {
        text: "Choose which game client's text to show",
        description: "Accessible name of the game-data server picker.",
    },
    "gamedataServer.followLanguage": {
        text: "Match language ({server})",
        description: "The picker's default entry: show game text from whichever client the display language reads. {server} is that client's name, e.g. 'Korea', so the reader sees what 'match' currently resolves to.",
    },
    "gamedataServer.name.en": {
        text: "Global",
        description: "The Global (English) Arknights client, run by Yostar. A game region name, not a language name.",
    },
    "gamedataServer.name.jp": {
        text: "Japan",
        description: "The Japanese Arknights client. A game region name, not a language name.",
    },
    "gamedataServer.name.kr": {
        text: "Korea",
        description: "The Korean Arknights client. A game region name, not a language name.",
    },
    "gamedataServer.name.cn": {
        text: "China",
        description: "The Chinese (mainland) Arknights client, run by Hypergryph. A game region name, not a language name.",
    },
    "gamedataServer.name.tw": {
        text: "Taiwan",
        description: "The Taiwanese (Traditional Chinese) Arknights client. A game region name, not a language name.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
