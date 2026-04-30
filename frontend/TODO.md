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

- [ ] **Higher-quality Myrtle logo.** Swap in a higher-resolution version of the site logo.

---

## `/operators` (List Page)

- [X] **Persist filter state.** Save filters in local storage so they survive navigating to an operator and back. Currently selecting "Defender", clicking into an operator, and returning resets the filter.

---

## `/operators/{id}` (Operator Detail Page)

- [X] **Remove the dot on the sidebar** in the operator page.
- [ ] **Trait/talent change view setting.** Add a Settings option to render trait/talent changes on a separate line, e.g.:

  ```
  Base Trait:
  Can attack all blocked enemies.

  Changes:
  ~~Can attack~~ Simultaneously attacks all blocked enemies.
  ```

- [ ] **Restyle the attack range block.** It currently looks like a button - try a different color (base orange?) and consider centering it.
- [ ] **Skin dialog: support scroll-wheel zoom** instead of only zoom buttons.
- [ ] **Update `descriptionToHtml`:** map blue highlight color to `#27e8e7`.

---

## Profile Page

- [ ] **Module titles.** Display module titles like `X` / `Y` instead of `N3`, etc. (See: [Discord reference](https://discord.com/channels/1461861870972436513/1461861872901951572/1495822322526060654))
- [ ] **Split the user dropdown into two controls.** The username and avatar should be a direct "view profile" link; keep the dropdown arrow as a separate button for the menu.
- [ ] **Update "copy username"** to include the discriminator/numbers.
- [ ] **Fix unowned operators display.**
- [ ] **Fix trust calculation** for the user roster.
- [ ] **Optimize operator visuals loading.** Profile-section operator art loads slowly - likely because the site fetches the full multi-MB E2 PNG for every operator. Options:
  - [ ] Use face-crop images (matches how the game itself handles roster views).
  - [ ] Or, if keeping full E2 art, serve a compressed version.
  - [ ] Or strip transparency by converting to JPG and storing separately (scriptable in bulk).

  This should reduce backend load and noticeably improve frontend responsiveness.

---

## `/tier-lists`

- [ ] **Remove the hover arrow** on tier-list cards - it's not needed.
- [ ] **Auto-delete empty tier lists** that have been empty for more than two weeks to reduce clutter.

## `/tier-lists/{id}`

- [ ] **Settings option to disable gray-out hover.**
- [ ] **Fix popup text** when clicking on a tier.

---

## `/leaderboard`

- [ ] **Configurable items per page.**

---

## `/tools/recruitment`

- [ ] **Layout: force-wrap "Starter".** Qualifications display is fine, but "Starter" sitting on the same line is awkward.
- [ ] **Default accordions to closed** in the result box when selecting tags (currently open by default).
- [ ] **Merge duplicate result combinations.** When different tag sets yield the *same* operator pool, combine them. Example: `Melee, Specialist, Debuff` and `Melee, Debuff` both yield only Waai Fu - these should be a single combined result with both tag combinations shown together.
- [ ] **Reverse lookup on hover/click.** When hovering or clicking an operator, show that operator's tag combinations.
- [ ] **Closing an accordion should close it completely** - verify this looks right.

---

## `/tools/dps`

- [ ] **Add character art** to operators in this tool.
- [ ] **Make the popup larger.**

---

## Pull Stats / Statistics

- [ ] **Show 3★ and 4★ rates** in addition to 5★/6★.
- [ ] **Better deviance calculation.** The current "above/below expected by X%" framing is misleading because it uses simple subtraction. Example: a 6★ rate of 4% (vs. 2% expected) shows as "above by 2%" - but that's *double* the expected rate. Conversely, a 5★ rate of 6% (vs. 8% expected) shows as "below by 2%" but is missing a quarter of the advertised rate.

  Switch to a percentage-of-expected calculation, displayed as either:
  - "X% of the expected rate", or
  - "above/below expected by X%" with the percentage scaled to expected.

- [ ] **Community average comparison.** Similar to how expected rate is shown and confirmed, also show the community average and compare against it.

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
