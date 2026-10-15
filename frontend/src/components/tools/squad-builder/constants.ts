/**
 * Every number and list the Squad Builder page picks itself rather than reads
 * from the game data, each with why. Game facts never live here: skill ranks,
 * masteries, module ranks, level caps and potentials come from the operator
 * index (`skillRules`, `phaseMaxLevels`, `modules[].rankCount`,
 * `potentialRankCount`), and the stage threat thresholds from
 * `GET /squads/rules`. The backend twin is `backend/src/core/squad/policy.rs`.
 */

/** Seats in a squad: the game client's constant, in no excel table. Mirrors `policy::SQUAD_SIZE`. */
export const SQUAD_SIZE = 12;
/** The index the analysis reports the support seat as, the first past the squad. */
export const SUPPORT_SEAT = 12;
/** Saved squads per account, a product choice. Mirrors `policy::SLOT_LIMIT`. */
export const SLOT_LIMIT = 4;
/** Longest squad name stored; `user_squads.name varchar(40)`. */
export const NAME_MAX_CHARS = 40;

/** The Economy stepper's floor. */
export const START_DP_MIN = 0;
/** The Economy stepper's ceiling, a product choice: two digits. A picked stage's `initialCost` (up to 999 in the level files) bypasses it, and the backend accepts up to `policy::START_DP_MAX` (999). */
export const START_DP_MAX = 99;
/** Starting DP before a stage is picked; a picked stage's `initialCost` replaces it. */
export const DEFAULT_START_DP = 10;

/** A dragged DP line or a held stepper settles this long before the next analysis goes out. */
export const REQUEST_DEBOUNCE_MS = 250;

/** The DPS calculator holds four cards: the seat itself plus this many squadmates. */
export const DPS_SQUADMATE_LIMIT = 3;

/** An enemy whose leak costs more than this many life points is a multi-life threat: "more than one", a definition. */
export const MULTI_LIFE_ABOVE = 1;
