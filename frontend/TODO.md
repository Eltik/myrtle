# Myrtle TODO

---

## Authentication & Accounts

- [ ] **Session duration options.** Let users pick how long they stay logged in: today only, until tab close, one week, or one month.
- [ ] **Default "Stay Logged In" to checked**, but allow users to uncheck it before signing in.
- [ ] **Leaderboard opt-in notice.** On sign-in (likely first-time, but worth deciding), show a popup informing users they're automatically opted into the leaderboard, with a note that it can be disabled in Settings.

---

## Legal & Contact

- [ ] **GDPR contact email.** Add a contact email for GDPR-related requests (data deletion, etc.).
- [ ] **Privacy page:** Remove the "eye" icon - users mistake it for a close/dismiss control.

---

## General UI

- [X] **Higher-quality Myrtle logo.** Swap in a higher-resolution version of the site logo.

---

## `/operators` (List Page)

- [X] **Persist filter state.** Save filters in local storage so they survive navigating to an operator and back. Currently selecting "Defender", clicking into an operator, and returning resets the filter.

---

## `/operators/{id}` (Operator Detail Page)

- [X] **Remove the dot on the sidebar** in the operator page.
- [X] **Trait/talent change view setting.** Add a Settings option to render trait/talent changes on a separate line, e.g.:

  ```
  Base Trait:
  Can attack all blocked enemies.

  Changes:
  ~~Can attack~~ Simultaneously attacks all blocked enemies.
  ```

- [X] **Restyle the attack range block.** It currently looks like a button - try a different color (base orange?) and consider centering it.
- [X] **Skin dialog: support scroll-wheel zoom** instead of only zoom buttons.
- [X] **Update `descriptionToHtml`:** map blue highlight color to `#27e8e7`.
- [X] Attack ranges for skills

---

## `/user/{id}` (Profile Page)

- [X] **Reorder profile sections.** Show `Stats` → `Score` first, then `Roster` → `Inventory` below.
- [X] **Replace the three-dot menu next to Share with a "Copy UID" button.**
- [X] **Module titles.** Display module titles like `X` / `Y` instead of `N3`, etc. (See: [Discord reference](https://discord.com/channels/1461861870972436513/1461861872901951572/1495822322526060654))
- [X] **Split the user dropdown into two controls.** The username and avatar should be a direct "view profile" link; keep the dropdown arrow as a separate button for the menu.
- [X] **Update "copy username"** to include the discriminator/numbers.
- [X] **Fix unowned operators display.**
- [X] **Fix trust calculation** for the user roster.
- [ ] **Optimize operator visuals loading.** Profile-section operator art loads slowly - likely because the site fetches the full multi-MB E2 PNG for every operator. Options:
  - [ ] Use face-crop images (matches how the game itself handles roster views).
  - [ ] Or, if keeping full E2 art, serve a compressed version.
  - [ ] Or strip transparency by converting to JPG and storing separately (scriptable in bulk).

  This should reduce backend load and noticeably improve frontend responsiveness.

---

## `/user/search`

- [X] **Substring matching for usernames.** Searching `cr` should display usernames FIRST that start with `cr` (e.g. "Crimson"). Users that contain the string (e.g. `Wisecrack`) should be displayed, but only after. If possible, order alphabetically. Search algorithm should stay the same for the backend, just the frontend should update things.

---

## `/tier-lists`

- [ ] **Remove the hover arrow** on tier-list cards - it's not needed.
- [ ] **Auto-delete empty tier lists** that have been empty for more than two weeks to reduce clutter.

## `/tier-lists/{id}`

- [ ] **Settings option to disable gray-out hover.**
- [ ] **Fix popup text** when clicking on a tier.

---

## `/user/leaderboard`

- [X] **Make the entire row a single button** that navigates to the user's page.
- [X] **Hide server share for now** - comment out the code rather than deleting it.
- [X] **Remove the "Top 3 Overall" component entirely** - redundant with the main leaderboard rows.
- [ ] **Configurable items per page.**

---

## `/tools/recruitment`


---

## `/tools/dps`

---

## `/gacha/history`

- [ ] **Banner name layout.** Keep the banner name on a single line, and/or fetch the actual banner name rather than a truncated/generic label.
- [ ] **Limited & collab pity reset handling.** Pity does not carry over between limited/collab banners. Verify whether a new limited banner pull is treated as the same banner as before, and **add a notification on collab and limited banners explaining that pity does not carry over**.

---

## Pull Stats / Statistics


---

## Data Export

- [ ] **Excel export.** Allow users to export operators and other roster data as an `.xlsx` spreadsheet.

---

## Roster Scoring (Power Level Algorithm)

Concept for an investment-aware power score (credit: Ghost):

- [ ] **Baseline:** power level of 1 at E0 Lv1.
- [ ] **Stats scaling:** take the percentage stat increase from E0 Lv1 → E2 Lv90 and apply it to the base value. (Will need careful weighting across HP, DEF, ATK, etc.)
- [ ] **Skills:** factor in skill level. SL7 might bump power by ~50% (e.g., from 5 → 7.5).
- [ ] **Masteries:** ~30% increase, but account for the fact that an operator can only use one skill at a time - don't double-count across S1/S2/S3 masteries.
- [ ] **Modules, potential, etc.** also feed into the score.

The goal is a single number that reflects how invested an operator is and how strong they currently are.

---
