/**
 * Whether the account's "Store gacha history" switch (`store_gacha`, Settings >
 * Privacy) is off, so a refresh would store nothing and the backend refuses it
 * with a 409.
 *
 * Only an explicit `false` is off. `null` is a profile with no settings row,
 * which the backend reads as the column default, on
 * (`queries::gacha::store_gacha_enabled`), so the refresh stays available.
 */
export function gachaStorageOff(user: { store_gacha?: boolean | null } | null | undefined): boolean {
    return user?.store_gacha === false;
}
