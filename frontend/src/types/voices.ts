/**
 * Voice types - re-exported from the ts-rs bindings generated out of
 * `backend/src/core/gamedata/types/voice.rs`.
 *
 * These were hand-written until a field renamed on one side only (`voiceUrl` ->
 * `voiceURL`, in the TypeScript alone) silently killed every voice line on both
 * servers. Nothing caught it: the payload arrives through an unchecked cast, the
 * field is nullable, and every consumer guards with `if (!url) return`.
 *
 * The `I`-prefixed aliases are kept so existing imports are unaffected. Do not
 * add fields here - change the Rust struct and re-run:
 *   cd backend && cargo test export_bindings
 *
 * Note the `/static/voices` and `/voices/:id` endpoints are consumed RAW, with
 * no `deepCamelize` (unlike the operator endpoints), so these generated types
 * are the wire format exactly.
 */
export type { LangType } from "./generated/LangType";
export type { PlaceType } from "./generated/PlaceType";
export type { UnlockType } from "./generated/UnlockType";
export type { VoiceType } from "./generated/VoiceType";

import type { CharExtraWord } from "./generated/CharExtraWord";
import type { ExtraVoiceConfigData } from "./generated/ExtraVoiceConfigData";
import type { FesTimeData } from "./generated/FesTimeData";
import type { FesTimeInterval } from "./generated/FesTimeInterval";
import type { FesVoiceData } from "./generated/FesVoiceData";
import type { FesVoiceWeight } from "./generated/FesVoiceWeight";
import type { StartTimeWithType } from "./generated/StartTimeWithType";
import type { UnlockParam } from "./generated/UnlockParam";
import type { Voice } from "./generated/Voice";
import type { VoiceData } from "./generated/VoiceData";
import type { VoiceLang } from "./generated/VoiceLang";
import type { VoiceLangDictEntry } from "./generated/VoiceLangDictEntry";
import type { VoiceLangGroupType } from "./generated/VoiceLangGroupType";
import type { VoiceLangTypeInfo } from "./generated/VoiceLangTypeInfo";
import type { Voices } from "./generated/Voices";

export type IUnlockParam = UnlockParam;
export type IVoiceData = VoiceData;
export type ICharExtraWord = CharExtraWord;
export type IVoiceLangTypeInfo = VoiceLangTypeInfo;
export type IVoiceLangGroupType = VoiceLangGroupType;
export type IStartTimeWithType = StartTimeWithType;
export type IFesTimeInterval = FesTimeInterval;
export type IFesTimeData = FesTimeData;
export type IFesVoiceData = FesVoiceData;
export type IFesVoiceWeight = FesVoiceWeight;
export type IExtraVoiceConfigData = ExtraVoiceConfigData;
export type IVoiceLangDictEntry = VoiceLangDictEntry;
export type IVoiceLang = VoiceLang;
export type IVoice = Voice;
export type IVoices = Voices;
