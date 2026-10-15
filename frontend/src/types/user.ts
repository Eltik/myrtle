/**
 * ts-rs bindings for `backend/src/database/models/user.rs`. These endpoints serve the
 * DB rows directly (snake_case, no `rename_all`), so the generated types are the wire
 * format. To change a field, edit the Rust struct and run `bun run gen:types`.
 */
import type { UserCheckin } from "./generated/UserCheckin";
import type { UserProfile } from "./generated/UserProfile";

export type IUserProfile = UserProfile;

/** Daily sign-in state from the game's `checkIn` section (`/get-user-checkin`). */
export type IUserCheckin = UserCheckin;
