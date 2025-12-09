import type { Operator } from "../impl/operator";

export type OperatorFromList = Pick<Operator, "id" | "name" | "nationId" | "groupId" | "teamId" | "position" | "isSpChar" | "rarity" | "profession" | "subProfessionId" | "profile" | "artists" | "portrait">;
