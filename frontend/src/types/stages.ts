/**
 * Stage, zone, activity and retro types - re-exported from the ts-rs bindings
 * generated out of `backend/src/core/gamedata/types/{stage,zone,activity,retro,
 * stage_index}.rs` and `backend/src/app/routes/stages.rs`.
 *
 * These endpoints (`/static/stages`, `/static/zones`, `/static/activities`,
 * `/static/retro_acts`, `/static/stage-index`) are consumed RAW - no
 * `deepCamelize` - and the Rust structs are already `rename_all = "camelCase"`,
 * so the generated types are the wire format exactly.
 *
 * Do not add fields here. Edit the Rust struct and run `bun run gen:types`.
 */
export type { AppearanceStyle } from "./generated/AppearanceStyle";
export type { StageDifficulty } from "./generated/StageDifficulty";
export type { StageType } from "./generated/StageType";
export type { ZoneType } from "./generated/ZoneType";

import type { ActivityBasicInfo } from "./generated/ActivityBasicInfo";
import type { DisplayDetailReward } from "./generated/DisplayDetailReward";
import type { RetroAct } from "./generated/RetroAct";
import type { Stage } from "./generated/Stage";
import type { StageClearDto } from "./generated/StageClearDto";
import type { StageDropInfo } from "./generated/StageDropInfo";
import type { StageUnlockCondition } from "./generated/StageUnlockCondition";
import type { Zone } from "./generated/Zone";

export type IUnlockCondition = StageUnlockCondition;
export type IDisplayDetailReward = DisplayDetailReward;
export type IStageDropInfo = StageDropInfo;
export type IStage = Stage;
export type IZone = Zone;
export type IActivity = ActivityBasicInfo;
export type IRetroAct = RetroAct;

/** One user's clear record for a stage (`GET /stages/clears`), not game data. */
export type IStageClear = StageClearDto;
export type StageClearsMap = Record<string, IStageClear>;
